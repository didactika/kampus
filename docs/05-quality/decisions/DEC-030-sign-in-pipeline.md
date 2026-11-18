---
id: DEC-030
status: proposed
---

# DEC-030 One sign-in pipeline, safe linking of external identities, secrets never stored in clear

- Date: 2026-11-18
- Context: Accounts are reached by password, passkey and identity providers; each method that skips a check (status, throttling, second factor) becomes the weak door. Tokens and secrets stored in clear turn a database leak into account takeovers.
- Options considered: one code path per method; delegating everything to an external identity server; one pipeline with pluggable credential steps (chosen).
- Decision:
  - Every method runs the same pipeline: identify, status, throttling, credential, second factor, session.
  - External identities link by issuer and subject; linking by email needs a verified claim and the campus policy, or the owner's confirmation.
  - Passwords use Argon2id with a breached-password check; passwords are never generated or sent; new accounts get a one-time set-password link.
  - One-time tokens are random, hashed, expiring and single-use; provider secrets and TOTP secrets are encrypted at rest.
  - Throttling is on by default, per account and per address, and answers never reveal whether an account exists.
- Consequences: No method is weaker than another; a database copy does not expose usable tokens or secrets; SAML and LDAP arrive later as new steps of the same pipeline.
- Affected artifacts: `crates/kampus-identity`, the sign-in screens, the operator and administrator guides.
