---
id: DEC-055
status: accepted
---

# DEC-055 Declared, typed, versioned events; deliveries with a dead-letter list; the audit log apart

- Date: 2026-11-18
- Context: Events without schemas break listeners silently; observers whose failures are swallowed lose work; a log that mixes audit and operational noise is neither trustworthy nor useful.
- Options considered: free-form events and one log table; an external broker for everything; declared events with an outbox on PostgreSQL (DEC-016) and a separate append-only audit log (chosen).
- Decision:
  - Every event type is declared with a versioned JSON schema; invalid payloads fail the transaction.
  - Events go to the outbox in the change's transaction and are dispatched after commit, at least once, ordered per object.
  - Each listener delivery has a state, retries with back-off and a dead-letter list that operators can retry.
  - Audited types write an append-only audit entry in the same transaction; visibility follows the object's permissions.
  - Retention is a per-campus policy; personal fields are declared per type for the privacy handlers.
- Consequences: Listeners can trust payloads; failures are visible and retryable; the audit log answers who did what without noise.
- Affected artifacts: `crates/events`.
