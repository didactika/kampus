---
id: DEC-063
status: accepted
---

# DEC-063 Declared routes, API clients with scopes intersected with permissions, OAuth 2 with PKCE and rate limits

- Date: 2026-11-18
- Context: An API whose functions are listed by hand drifts from the code; tokens that grant everything their account can do are dangerous; integrations without limits can take the campus down.
- Options considered: hand-written route lists; tokens equal to their account; declared routes with scopes intersected with permissions (chosen).
- Decision:
  - Modules declare routes with their permission, API scope and schemas; the OpenAPI 3.1 document is generated from them.
  - A token reaches a route only with the route's scope and the account's permission in the target scope.
  - Third-party applications use OAuth 2 authorisation code with PKCE, short access tokens and rotating refresh tokens; reuse revokes the family.
  - Rate limits per token and address on the cache port, answered with `429` and `Retry-After`.
  - Versions are paths; deprecations announce `Deprecation` and `Sunset` headers.
- Consequences: The document always matches the code; integrations get least privilege; one integration cannot exhaust the campus.
- Affected artifacts: `crates/api`, every module's routes, the integrator guide.
