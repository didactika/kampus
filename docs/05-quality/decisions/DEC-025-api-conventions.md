---
id: DEC-025
status: accepted
---

# DEC-025 REST conventions, cursor pagination and additive versioning

- Date: 2026-11-18
- Context: The API is the only way into Kampus for the web client, integrators and plugins (RF-103); it must be predictable, documented and stable across releases.
- Options considered: GraphQL first; REST with free conventions per module; REST with shared conventions described by OpenAPI (chosen; GraphQL can come later on top, as planned for the extras).
- Decision:
  - OpenAPI 3.1 generated from the code; plural-noun paths under /api/v1; snake_case JSON; RFC 3339 times; string UUIDs and decimals.
  - Keyset pagination with opaque cursors (default 50, maximum 200); filters by field; sorting over declared fields with a stable tie-breaker; permissions filter before paging.
  - Errors are RFC 9457 problem details (DEC-018).
  - Additive changes only within a major version; deprecations with Deprecation and Sunset headers kept for at least two minor releases; breaking changes in a new major version served beside the old one.
  - Rate limits answer 429 with Retry-After; webhooks are signed and carry an event id.
- Consequences: One shape for every module; the web client's API client is generated; integrators can rely on a version for years; pagination stays fast on large tables.
- Affected artifacts: `crates/kampus-api`, every module's routes, `web/`, the integrator guide.
