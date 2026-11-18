---
id: DEC-026
status: accepted
---

# DEC-026 Optimistic versions, idempotency keys and rules enforced by the database

- Date: 2026-11-18
- Context: Concurrent edits, double submissions and retries are normal on the web and in integrations. Read-then-write checks let two requests cross a limit together, and retried requests create duplicates.
- Options considered: last write wins; pessimistic locks held across requests; optimistic versions with idempotency keys and database-enforced rules (chosen).
- Decision:
  - Every mutable resource has a version, exposed as an ETag; updates and deletes require If-Match, and a stale version is a precondition_failed error.
  - POST requests accept an Idempotency-Key; a repeated key returns the first result.
  - Limits and uniqueness are enforced by constraints or by a lock in the same transaction, never by reading first and inserting afterwards.
- Consequences: No lost updates, no duplicates from retries or double clicks, and no limit that two requests can cross together; clients must handle precondition_failed, which the generated client does.
- Affected artifacts: `crates/common`, `crates/api`, every module with limits or uniqueness rules, `web/`.
