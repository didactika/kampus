---
id: DATA-IDENTITY
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: identity and users

PostgreSQL schemas `identity` (owned by `kampus-identity`) and `users` (owned by `kampus-users`), following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [identity domain model](../../00-domain-model/identity.md). Names are logical; physically the schema carries the installation prefix, `kmp_identity` and `kmp_users` by default (DEC-039).

![Identity schema](./identity.svg)

## Schema `identity`

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `account` | `id` PK FK → `access.principal` (`CASCADE`), `campus_id` FK → `access.scope`, `scope_id` FK → `access.scope` `UNIQUE` (the user scope), `kind` (`person`, `service`), `username`, `email`, `email_verified_at`, `status`, `status_reason`, `status_changed_at`, `status_changed_by`, `last_sign_in_at`, `pseudonym`, `deletion_due_at`, `deleted_at`, `pending_email` (a new address waiting for its confirmation, step 2), `version`, `created_at` | `UNIQUE` (`campus_id`, `username`) and (`campus_id`, `email`): both are stored lower-case, so the keys compare case-insensitively (RN-022), and a tombstone frees them by setting them to `NULL`; `CHECK` status in the five states; `CHECK` a person has an email unless deleted; index on `deletion_due_at` for the deletion job |
| `account_reference` | External references of accounts (step 3), in the shape of the [conventions](conventions.md#external-references) | Read with `identity.account.read`, set with `identity.account.invite` |
| `account_transition` | `id`, `account_id` FK (`CASCADE`), `from_status`, `to_status`, `reason`, `actor` (the account that made the change, without a foreign key so history survives it), `at` | Append-only history of RN-023; reasons are erased with the account |
| `password` | `account_id` PK FK (`CASCADE`), `hash` (a PHC string), `algorithm_params`, `set_at`, `must_change` | One row per account; service accounts have none (RN-039) |
| `passkey` | `id`, `account_id` FK (`CASCADE`), `credential` `UNIQUE`, `public_key`, `sign_count`, `label`, `created_at`, `last_used_at` | WebAuthn credential, usable as first or second factor |
| `identity_provider` | `id`, `campus_id` FK → `access.scope`, `kind` (`oidc`; later `saml`, `ldap`), `issuer`, `client` (the client id at the provider), `client_secret_encrypted`, `claim_mapping` jsonb (validated schema), `linking_policy` (`subject_only`, `verified_email`, `confirm`), `created_at` | `UNIQUE` (`campus_id`, `issuer`) |
| `external_identity` | `id`, `account_id` FK (`CASCADE`), `provider_id` FK (`CASCADE`), `subject`, `email_at_link`, `linked_at`, `last_used_at` | `UNIQUE` (`provider_id`, `subject`) (RN-028) |
| `second_factor` | `id`, `account_id` FK (`CASCADE`), `kind` (`totp`, `passkey`, `email`), `secret_encrypted` (TOTP), `passkey_id` FK → `passkey`, `label`, `created_at`, `last_used_at` | `UNIQUE` (`account_id`, `label`); `CHECK` the column of its kind is set |
| `recovery_code` | `id`, `account_id` FK (`CASCADE`), `hash`, `created_at`, `used_at` | Generated in sets of ten; `UNIQUE` (`account_id`, `hash`) |
| `one_time_token` | `id`, `account_id` FK (`CASCADE`), `purpose` (`confirm_email`, `invitation`, `set_password`, `recovery`, `email_code`, `second_factor` for a sign-in waiting for its second factor, `email_change` from step 2), `hash` `UNIQUE`, `created_at`, `expires_at`, `used_at` | `CHECK` `expires_at` > `created_at`; index on `expires_at` for purging; issuing a token replaces the unused ones of the same purpose |
| `session` | `id`, `account_id` FK (`CASCADE`), `token_hash` `UNIQUE`, `mfa_level` (`none`, `second_factor`), `strong_auth_at`, `created_at`, `last_seen_at`, `expires_at`, `client`, `address` | Index (`account_id`); revocation is a delete by `account_id` (RN-031); `last_seen_at` is written at most once a minute |
| `api_token` | `id`, `account_id` FK (`CASCADE`), `name`, `prefix` `UNIQUE`, `hash`, `created_at`, `expires_at`, `last_used_at`, `revoked_at` | `CHECK` `expires_at` > `created_at`; scopes in `api_token_scope` |
| `api_token_scope` | `token_id` FK (`CASCADE`), `permission_code` FK → `access.permission`, `scope_id` FK → `access.scope` | PK (`token_id`, `permission_code`, `scope_id`) |
| `sign_in_attempt` | `id`, `campus_id` FK → `access.scope`, `account_id` FK (`SET NULL`, nullable), `name_tried_hash`, `address`, `outcome` (`success`, `failure`, `throttled`, `second_factor_required`), `at` | Indexes (`account_id`, `at`), (`campus_id`, `name_tried_hash`, `at`), (`campus_id`, `address`, `at`) for throttling, and (`at`) for purging after the retention period |

**Deletion keeps the row as a tombstone** (RN-034): the privacy handlers of every module erase their personal data, then the account row itself has `username`, `email` and every personal column set to `NULL`, `pseudonym` set, and status `deleted`. Its id never changes, so foreign keys from academic records and contributions stay valid. A `CHECK` requires `username` unless the status is `deleted`, and `pseudonym` when it is.

## Schema `users`

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `profile` | `account_id` PK FK → `identity.account` (`CASCADE`), `campus_id` FK → `access.scope`, `given_name`, `family_name`, `display_name`, `language_code` (a BCP 47 tag; its foreign key to `i18n.language` arrives with that module), `time_zone` (an IANA name, checked by the service), `version`, `updated_at` | Saved the first time it changes; reads empty before. `picture_file_id` arrives with the files module |
| `account_field` | `account_id` FK → `identity.account` (`CASCADE`), `field_value_id` FK → `customise.field_value` (`CASCADE`) | Arrives with the customise module: profile fields are custom fields of the entity kind `account` (DEC-070) |
| `preference` | `account_id` FK (`CASCADE`), `key` (declared by a module in its descriptor), `value` jsonb (checked against its declaration), `updated_at` | PK (`account_id`, `key`); undeclared keys are refused and a missing value reads its default |
| `system_group` | `id`, `scope_id` FK → `access.scope` (a campus or a category), `code`, `name`, `kind` (`manual`, `rule`), `rule` jsonb (a profile field and the value it equals), `owner_module`, `created_at` | `UNIQUE` (`scope_id`, `code`); `CHECK` a rule exactly when the kind is `rule` |
| `system_group_member` | `group_id` FK (`CASCADE`), `account_id` FK (`CASCADE`), `origin` (`manual`, `rule`, `sync`), `added_at` | PK (`group_id`, `account_id`, `origin`) (RN-037) |

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Secrets in clear text (reset tokens, confirmation secrets, provider secrets) | Hashes for tokens, encryption for secrets that must be read back |
| Linking external accounts by the first matching email | Unique (`provider_id`, `subject`); email linking only by policy |
| Lockout state in generic preferences | `sign_in_attempt` records with their own retention |
| Soft deletion that keeps personal data | Erasure plus tombstone with a pseudonym |
| Suspension that leaves sessions alive | Server-side sessions revoked with the status change |
| One wide user table for every field | Profile, typed custom fields, and preferences declared by modules |
