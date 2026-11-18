---
id: DEC-076
status: accepted
---

# DEC-076 A generated API client with ETag caching, conflict handling and idempotent mutations

- Date: 2026-11-18
- Context: Hand-written calls drift from the API, overwrite other people's changes silently and create duplicates on double clicks.
- Options considered: hand-written fetch calls; a GraphQL layer; a client generated from the OpenAPI document (chosen).
- Decision:
  - The typed client is generated from the OpenAPI document on every build; UI modules use only it.
  - Reads are cached per resource and revalidated with ETag; writes send `If-Match` and show conflicts to the person.
  - Mutations carry an `Idempotency-Key`; live updates arrive over a WebSocket filtered by permissions.
- Consequences: The client cannot call an operation that does not exist; concurrent edits are never lost silently.
- Affected artifacts: `web/`, `@kampus/sdk`, `crates/api`.
