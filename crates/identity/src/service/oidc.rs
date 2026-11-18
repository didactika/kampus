// ------------------------------------------------------------------------------
//
//     _  __    _    __  __ ____  _   _ ____
//    | |/ /   / \  |  \/  |  _ \| | | / ___|
//    | ' /   / _ \ | |\/| | |_) | | | \___ \
//    | . \  / ___ \| |  | |  __/| |_| |___) |
//    |_|\_\/_/   \_\_|  |_|_|    \___/|____/
//
//    Knowledge Academic Modular Platform for Universal Study
//
// ------------------------------------------------------------------------------
//
// This file is part of Kampus - https://github.com/didactika/kampus
//
// Kampus is free software: you can redistribute it and/or modify it under the terms of the
// GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with Kampus.
// If not, see <https://www.gnu.org/licenses/>.
//
// @package   kampus-identity
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! OpenID Connect providers (RN-028): a campus signs people in through its providers with the
//! authorization code flow, PKCE and a nonce, and the identity of the provider links to an account by
//! issuer and subject. A link by email needs the provider's verified email and the campus policy, or the
//! owner's confirmation; never the first account that shares the email.
//!
//! The state of a sign-in in progress travels sealed with the secret key, in a cookie the client keeps,
//! so nothing is stored before the provider answers. Redirects are never followed.

use kampus_access::{Actor, ScopeId};
use kampus_common::{Error, FieldError, Result};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::permissions::PROVIDER_MANAGE;
use crate::service::session::Client;
use crate::service::sign_in::{Outcome, SignInOutcome, name_hash};
use crate::{AccountKind, AccountStatus, Identity, LinkingPolicy, ProviderId, Session, sql};

/// How long a sign-in may stay at the provider, and how long a link waits for its confirmation.
const FLOW_LIFETIME: time::Duration = time::Duration::minutes(10);

/// How long a request to a provider may take.
const PROVIDER_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// A provider to configure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderConfig {
    /// The issuer URL; its discovery document gives the endpoints and keys.
    pub issuer: String,
    /// The client id Kampus has at the provider.
    pub client: String,
    /// The client secret, sealed when stored.
    pub client_secret: String,
    /// How identities that are not linked yet find their account.
    pub linking_policy: LinkingPolicy,
}

/// A provider as listed on the sign-in page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Provider {
    /// The provider.
    pub id: ProviderId,
    /// Its issuer.
    pub issuer: String,
    /// Its linking policy.
    pub linking_policy: LinkingPolicy,
}

/// The start of a sign-in at a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcStart {
    /// Where to send the browser.
    pub authorize_url: String,
    /// The sealed state of the sign-in, to keep in a cookie until the provider answers.
    pub flow: String,
}

/// What a provider's answer leads to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OidcOutcome {
    /// The identity is linked: the pipeline went on as for any method.
    SignedIn(SignInOutcome),
    /// The campus asks the owner to confirm the link: they sign in with their own credentials and call
    /// [`Identity::confirm_link`] with this sealed link.
    ConfirmLink {
        /// The sealed identity to link.
        link: String,
    },
}

/// The sealed state of a sign-in at a provider.
#[derive(Debug, Serialize, Deserialize)]
struct Flow {
    /// The provider.
    provider: uuid::Uuid,
    /// The state the provider must send back.
    state: String,
    /// The nonce the ID token must carry.
    nonce: String,
    /// The PKCE verifier.
    verifier: String,
    /// Where the provider sends the browser back.
    redirect: String,
    /// When the flow expires, in Unix seconds.
    expires: i64,
}

/// A sealed identity waiting for its owner's confirmation.
#[derive(Debug, Serialize, Deserialize)]
struct Link {
    /// The provider.
    provider: uuid::Uuid,
    /// The subject at the provider.
    subject: String,
    /// The email the provider gave.
    email: Option<String>,
    /// When the link expires, in Unix seconds.
    expires: i64,
}

/// The identity a provider asserted.
struct Asserted {
    /// The subject.
    subject: String,
    /// The email, lower-case.
    email: Option<String>,
    /// Whether the provider verified the email.
    email_verified: bool,
}

/// A provider as stored.
struct Stored {
    /// The campus it belongs to.
    campus: ScopeId,
    /// The issuer.
    issuer: String,
    /// The client id.
    client: String,
    /// The client secret, opened.
    secret: String,
    /// The linking policy.
    linking_policy: LinkingPolicy,
}

/// The error of a flow or a link that is unknown, changed or expired.
fn invalid_flow() -> Error {
    Error::validation(
        "identity.oidc_flow_invalid",
        vec![FieldError::new("state", "valid")],
    )
}

/// The error of a provider that cannot be reached or answers wrongly.
fn provider_failed(error: &(dyn std::error::Error + 'static)) -> Error {
    let mut cause = error.to_string();
    let mut source = error.source();
    while let Some(inner) = source {
        cause.push_str(": ");
        cause.push_str(&inner.to_string());
        source = inner.source();
    }
    Error::unavailable("identity.oidc_provider_failed").with_param("cause", cause)
}

/// The answer to an identity that finds no account.
fn not_linked() -> Error {
    Error::unauthenticated("identity.external_identity_not_linked")
}

impl Identity {
    /// Configures an identity provider for a campus; needs `identity.provider.manage` there.
    ///
    /// # Errors
    ///
    /// `permission_denied` without the permission, `validation` for an issuer that is not a URL,
    /// `conflict` when the campus has a provider with that issuer.
    pub async fn add_provider(
        &self,
        actor: &Actor,
        campus: ScopeId,
        config: ProviderConfig,
    ) -> Result<ProviderId> {
        self.access.require(actor, PROVIDER_MANAGE, campus).await?;
        let issuer = IssuerUrl::new(config.issuer.trim().to_owned()).map_err(|_| {
            Error::validation(
                "identity.oidc_issuer_invalid",
                vec![FieldError::new("issuer", "url")],
            )
        })?;
        let id = ProviderId::generate();
        sqlx::query(sql(format!(
            "insert into {} (id, campus_id, kind, issuer, client, client_secret_encrypted, claim_mapping, linking_policy, created_at)
             values ($1, $2, 'oidc', $3, $4, $5, '{{}}'::jsonb, $6, $7)",
            self.table("identity_provider")
        )))
        .bind(id.as_uuid())
        .bind(campus.as_uuid())
        .bind(issuer.as_str())
        .bind(config.client.trim())
        .bind(self.key.seal(config.client_secret.as_bytes())?)
        .bind(config.linking_policy.as_str())
        .bind(self.now().as_offset_date_time())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .map_err(|error| {
            if error.code() == "db.unique_violation" {
                Error::conflict("identity.provider_exists")
            } else {
                error
            }
        })?;
        Ok(id)
    }

    /// The providers of a campus, for its sign-in page.
    ///
    /// # Errors
    ///
    /// `unavailable` when the database fails.
    pub async fn providers(&self, campus: ScopeId) -> Result<Vec<Provider>> {
        let rows = sqlx::query(sql(format!(
            "select id, issuer, linking_policy from {} where campus_id = $1 order by created_at, id",
            self.table("identity_provider")
        )))
        .bind(campus.as_uuid())
        .fetch_all(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        rows.iter()
            .map(|row| {
                Ok(Provider {
                    id: ProviderId::from_uuid(row.get("id")),
                    issuer: row.get("issuer"),
                    linking_policy: row.get::<String, _>("linking_policy").parse()?,
                })
            })
            .collect()
    }

    /// Starts a sign-in at a provider: the browser goes to the returned URL and comes back to `redirect`
    /// with a code and the state.
    ///
    /// # Errors
    ///
    /// `not_found` for an unknown provider, `validation` for a redirect that is not a URL, `unavailable`
    /// when the provider cannot be discovered.
    pub async fn oidc_start(&self, provider: ProviderId, redirect: &str) -> Result<OidcStart> {
        let stored = self.stored_provider(provider).await?;
        let client = self.oidc_client(&stored, redirect).await?;
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_owned()))
            .add_scope(Scope::new("profile".to_owned()))
            .set_pkce_challenge(challenge)
            .url();
        let flow = Flow {
            provider: provider.as_uuid(),
            state: state.secret().clone(),
            nonce: nonce.secret().clone(),
            verifier: verifier.secret().clone(),
            redirect: redirect.to_owned(),
            expires: self.expiry(),
        };
        Ok(OidcStart {
            authorize_url: url.to_string(),
            flow: self.seal_json(&flow)?,
        })
    }

    /// Finishes a sign-in at a provider with the code and state it sent back and the sealed flow: the ID
    /// token is verified, the identity finds its account by issuer and subject (RN-028), and the pipeline
    /// goes on as for any method (RN-024).
    ///
    /// # Errors
    ///
    /// `validation` for a flow that is unknown, expired or of another state, `unavailable` when the
    /// provider fails or its token does not verify, `rate_limited` while throttled, `unauthenticated` when
    /// the identity finds no active account.
    pub async fn oidc_callback(
        &self,
        flow: &str,
        code: &str,
        state: &str,
        client: &Client,
    ) -> Result<OidcOutcome> {
        let flow: Flow = self.open_json(flow)?;
        if flow.expires < self.now().as_offset_date_time().unix_timestamp()
            || !crate::support::secret::same(flow.state.as_bytes(), state.as_bytes())
        {
            return Err(invalid_flow());
        }
        let provider = ProviderId::from_uuid(flow.provider);
        let stored = self.stored_provider(provider).await?;
        let asserted = self.exchange(&stored, &flow, code).await?;
        let policy = self.policies.policy(stored.campus).await?;
        let hash = name_hash(stored.campus, &format!("oidc:{provider}:{}", asserted.subject));
        let address = client.address.as_deref();

        let linked = self.linked_account(provider, &asserted.subject).await?;
        if let Some(retry_after) = self
            .throttled(stored.campus, &policy, linked, &hash, address)
            .await?
        {
            self.record_attempt(stored.campus, linked, &hash, address, Outcome::Throttled)
                .await?;
            return Err(crate::service::sign_in::throttled_error(retry_after));
        }
        let asserted_subject = asserted.subject.clone();
        let account = match linked {
            Some(id) => Some(self.read_account(id).await?),
            None => match stored.linking_policy {
                LinkingPolicy::SubjectOnly => None,
                LinkingPolicy::VerifiedEmail => self.link_by_email(provider, &stored, &asserted).await?,
                LinkingPolicy::Confirm => {
                    let link = Link {
                        provider: flow.provider,
                        subject: asserted.subject,
                        email: asserted.email,
                        expires: self.expiry(),
                    };
                    return Ok(OidcOutcome::ConfirmLink {
                        link: self.seal_json(&link)?,
                    });
                }
            },
        };
        let Some(account) =
            account.filter(|a| a.status == AccountStatus::Active && a.kind == AccountKind::Person)
        else {
            self.record_attempt(stored.campus, linked, &hash, address, Outcome::Failure)
                .await?;
            return Err(not_linked());
        };
        sqlx::query(sql(format!(
            "update {} set last_used_at = $3 where provider_id = $1 and subject = $2",
            self.table("external_identity")
        )))
        .bind(provider.as_uuid())
        .bind(&asserted_subject)
        .bind(self.now().as_offset_date_time())
        .execute(self.db.pool())
        .await
        .map_err(kampus_db::classify)?;
        self.conclude(&account, &policy, client, &hash)
            .await
            .map(OidcOutcome::SignedIn)
    }

    /// Links a provider identity to the session's account once its owner confirms it (RN-028); needs a
    /// recent strong authentication (RN-032), since a link is a new way in.
    ///
    /// # Errors
    ///
    /// `validation` for a link that is unknown, expired or of another campus, `permission_denied` without
    /// a recent strong authentication, `conflict` when the identity is linked to an account already.
    pub async fn confirm_link(&self, session: &Session, link: &str) -> Result<()> {
        let policy = self.policies.policy(session.campus).await?;
        self.require_recent_strong_auth(session, &policy)?;
        let link: Link = self.open_json(link)?;
        if link.expires < self.now().as_offset_date_time().unix_timestamp() {
            return Err(invalid_flow());
        }
        let stored = self.stored_provider(ProviderId::from_uuid(link.provider)).await?;
        if stored.campus != session.campus {
            return Err(invalid_flow());
        }
        let account = self.read_account(session.account).await?;
        self.insert_link(
            ProviderId::from_uuid(link.provider),
            &link.subject,
            link.email.as_deref(),
            account.id,
        )
        .await?;
        self.tell_owner(&account, crate::Notice::IdentityLinked).await;
        Ok(())
    }

    /// Links the identity to the active account with the provider's verified email, when there is one.
    async fn link_by_email(
        &self,
        provider: ProviderId,
        stored: &Stored,
        asserted: &Asserted,
    ) -> Result<Option<crate::Account>> {
        let (true, Some(email)) = (asserted.email_verified, asserted.email.as_deref()) else {
            return Ok(None);
        };
        let mut conn = self.db.pool().acquire().await.map_err(kampus_db::classify)?;
        let account = self.find_in(&mut conn, stored.campus, email).await?;
        drop(conn);
        let Some(account) = account.filter(|a| {
            a.email.as_deref() == Some(email)
                && a.status == AccountStatus::Active
                && a.kind == AccountKind::Person
        }) else {
            return Ok(None);
        };
        self.insert_link(provider, &asserted.subject, Some(email), account.id)
            .await?;
        Ok(Some(account))
    }

    /// Stores a link between a provider identity and an account.
    async fn insert_link(
        &self,
        provider: ProviderId,
        subject: &str,
        email: Option<&str>,
        account: crate::AccountId,
    ) -> Result<()> {
        sqlx::query(sql(format!(
            "insert into {} (id, account_id, provider_id, subject, email_at_link, linked_at) values ($1, $2, $3, $4, $5, $6)",
            self.table("external_identity")
        )))
        .bind(uuid::Uuid::now_v7())
        .bind(account.as_uuid())
        .bind(provider.as_uuid())
        .bind(subject)
        .bind(email)
        .bind(self.now().as_offset_date_time())
        .execute(self.db.pool())
        .await
        .map(drop)
        .map_err(kampus_db::classify)
        .map_err(|error| {
            if error.code() == "db.unique_violation" {
                Error::conflict("identity.external_identity_linked")
            } else {
                error
            }
        })
    }

    /// The account a provider identity is linked to.
    async fn linked_account(&self, provider: ProviderId, subject: &str) -> Result<Option<crate::AccountId>> {
        sqlx::query_scalar(sql(format!(
            "select account_id from {} where provider_id = $1 and subject = $2",
            self.table("external_identity")
        )))
        .bind(provider.as_uuid())
        .bind(subject)
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)
        .map(|id| id.map(crate::AccountId::from_uuid))
    }

    /// Exchanges the code and verifies the ID token with the provider's keys, audience and nonce.
    async fn exchange(&self, stored: &Stored, flow: &Flow, code: &str) -> Result<Asserted> {
        let client = self.oidc_client(stored, &flow.redirect).await?;
        let http = Self::http()?;
        let token = client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .map_err(|e| provider_failed(&e))?
            .set_pkce_verifier(PkceCodeVerifier::new(flow.verifier.clone()))
            .request_async(&http)
            .await
            .map_err(|e| provider_failed(&e))?;
        let id_token = token.id_token().ok_or_else(|| {
            Error::unavailable("identity.oidc_provider_failed")
                .with_param("cause", "the provider sent no ID token")
        })?;
        let claims = id_token
            .claims(&client.id_token_verifier(), &Nonce::new(flow.nonce.clone()))
            .map_err(|e| provider_failed(&e))?;
        Ok(Asserted {
            subject: claims.subject().as_str().to_owned(),
            email: claims.email().map(|e| e.as_str().trim().to_lowercase()),
            email_verified: claims.email_verified().unwrap_or(false),
        })
    }

    /// A client for a provider, from its discovery document.
    async fn oidc_client(
        &self,
        stored: &Stored,
        redirect: &str,
    ) -> Result<
        CoreClient<
            openidconnect::EndpointSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointNotSet,
            openidconnect::EndpointMaybeSet,
            openidconnect::EndpointMaybeSet,
        >,
    > {
        let issuer = IssuerUrl::new(stored.issuer.clone()).map_err(|e| provider_failed(&e))?;
        let metadata = CoreProviderMetadata::discover_async(issuer, &Self::http()?)
            .await
            .map_err(|e| provider_failed(&e))?;
        let redirect = RedirectUrl::new(redirect.to_owned()).map_err(|_| {
            Error::validation(
                "identity.oidc_redirect_invalid",
                vec![FieldError::new("redirect", "url")],
            )
        })?;
        Ok(CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(stored.client.clone()),
            Some(ClientSecret::new(stored.secret.clone())),
        )
        .set_redirect_uri(redirect))
    }

    /// The HTTP client for providers: no redirects, a timeout.
    fn http() -> Result<openidconnect::reqwest::Client> {
        openidconnect::reqwest::Client::builder()
            .redirect(openidconnect::reqwest::redirect::Policy::none())
            .timeout(PROVIDER_TIMEOUT)
            .build()
            .map_err(|e| provider_failed(&e))
    }

    /// A provider as stored, with its secret opened.
    async fn stored_provider(&self, provider: ProviderId) -> Result<Stored> {
        let row = sqlx::query(sql(format!(
            "select campus_id, issuer, client, client_secret_encrypted, linking_policy from {} where id = $1",
            self.table("identity_provider")
        )))
        .bind(provider.as_uuid())
        .fetch_optional(self.db.pool())
        .await
        .map_err(kampus_db::classify)?
        .ok_or_else(|| Error::not_found("identity.provider_not_found"))?;
        let sealed: Vec<u8> = row.get("client_secret_encrypted");
        Ok(Stored {
            campus: ScopeId::from_uuid(row.get("campus_id")),
            issuer: row.get("issuer"),
            client: row.get("client"),
            secret: String::from_utf8(self.key.open(&sealed)?)
                .map_err(|_| Error::internal("identity.sealed_secret_unreadable"))?,
            linking_policy: row.get::<String, _>("linking_policy").parse()?,
        })
    }

    /// When a flow or link made now expires, in Unix seconds.
    fn expiry(&self) -> i64 {
        (self.now().as_offset_date_time() + FLOW_LIFETIME).unix_timestamp()
    }

    /// Seals a value as JSON, in URL-safe base64.
    fn seal_json(&self, value: &impl Serialize) -> Result<String> {
        use base64::Engine;
        let json =
            serde_json::to_vec(value).map_err(|e| Error::internal("identity.seal_failed").with_source(e))?;
        Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(self.key.seal(&json)?))
    }

    /// Opens a value sealed by [`Identity::seal_json`].
    fn open_json<T: for<'de> Deserialize<'de>>(&self, sealed: &str) -> Result<T> {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(sealed.trim())
            .map_err(|_| invalid_flow())?;
        let json = self.key.open(&bytes).map_err(|_| invalid_flow())?;
        serde_json::from_slice(&json).map_err(|_| invalid_flow())
    }
}
