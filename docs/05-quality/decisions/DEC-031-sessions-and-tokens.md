---
id: DEC-031
status: accepted
---

# DEC-031 Server-side sessions checked on every request, central revocation, scoped API tokens

- Date: 2026-11-18
- Context: Suspending a person or changing a password must take effect at once, on every device and integration.
- Options considered: stateless signed tokens for sessions; sessions checked only at sign-in; server-side sessions checked on every request (chosen).
- Decision:
  - Sessions are stored server-side (database, cached) with their second-factor level and the time of the last strong authentication.
  - Every request checks the session and the account status; suspension, deletion or a password change revokes all sessions and API tokens of the account.
  - Sensitive operations need a strong authentication at most 15 minutes old (step-up).
  - API tokens are scoped to permissions and scopes, expire (at most one year), are shown once, stored hashed and identified by a prefix.
- Consequences: A suspended person loses access immediately everywhere; integrations hold only what they need, for a limited time; one cache lookup per request.
- Affected artifacts: `crates/kampus-identity`, `crates/kampus-api`, the integrator guide.
