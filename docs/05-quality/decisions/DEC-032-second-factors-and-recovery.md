---
id: DEC-032
status: accepted
---

# DEC-032 Second factors in the pipeline, and recovery that asks for them

- Date: 2026-11-18
- Context: A second factor is useless if recovery by email resets the password and signs the person in without it.
- Options considered: second factors as an optional add-on after sign-in; recovery by email only; second factors inside the pipeline with recovery that requires them (chosen).
- Decision:
  - Passkeys (WebAuthn), authenticator apps (TOTP) and email codes; recovery codes shown once and stored hashed.
  - A second factor is required when the person enabled it, when a role requires it in a scope where it is granted, or when the campus requires it, for every client including the API sessions of the web client.
  - Recovery: email token, then a second factor or a recovery code, then the new credential; all sessions are revoked and the person signs in normally afterwards.
- Consequences: Controlling someone's email is not enough to take over an account protected by a second factor; roles with risky permissions can require second factors where they matter.
- Affected artifacts: `crates/identity`, the security screens, the handbook.
