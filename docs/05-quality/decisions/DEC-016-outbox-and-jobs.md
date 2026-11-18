---
id: DEC-016
status: accepted
---

# DEC-016 Transactional outbox and durable jobs on the database

- Date: 2026-11-18
- Context: events, listeners, notifications and background work must never be lost, never fire for a change that was rolled back, and never fail without anyone seeing it. A message broker adds a system to operate and still needs an outbox to be consistent with the database.
- Options considered: in-process listeners inside the transaction; a message broker (RabbitMQ, NATS) from the first release; an outbox and a job queue on the database, with a broker adapter later behind the task queue port (chosen).
- Decision:
  - Domain events are written to an **outbox table in the same transaction** as the change, and dispatched after commit.
  - Listeners run after commit, are idempotent (each delivery has an id), retry with back-off, and end in a visible **dead-letter** state that administrators can replay.
  - Jobs (deferred, long and scheduled work, mail and notifications) live in a **durable queue on the database**, claimed with `FOR UPDATE SKIP LOCKED`, with retries, a final failed state with its error, metrics and alerts. Only one scheduler is active, by database lease.
  - The queue is a port (DEC-009): Redis, Valkey or NATS adapters can replace the database queue when an installation needs it.
- Consequences: no event or mail is lost when a request or a worker dies; no extra system in small installations; a failure always leaves a record. The outbox and queue tables are part of `kampus-events` and `kampus-tasks`, with their own conformance tests.
- Affected artifacts: `crates/kampus-events`, `crates/kampus-tasks`, every listener and job, the operator guide.
