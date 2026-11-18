---
id: DEC-060
status: accepted
---

# DEC-060 Jobs with leases, retries and concurrency keys; schedules with unique ticks

- Date: 2026-11-18
- Context: DEC-016 puts jobs on the database. Locks that a dead process never releases, scheduled tasks started twice by two nodes and tasks that fail without anyone seeing it are the usual failures of background work.
- Options considered: advisory locks held for the whole run; one scheduler node elected; leases renewed while working and ticks unique per schedule and instant (chosen).
- Decision:
  - Job types are declared with a payload schema, retry policy, timeout and an optional concurrency key.
  - Workers claim with `FOR UPDATE SKIP LOCKED` and renew a lease; an expired lease frees the job.
  - Retries use exponential back-off; after the last one the job is failed, alerted and retryable by hand.
  - A schedule tick is unique per schedule and instant, inserted with `ON CONFLICT DO NOTHING`; any number of schedulers may run.
  - Long jobs report progress and honour cancellation between steps; payloads hold identifiers, not personal data.
- Consequences: No stuck locks, no double runs, no silent failures; the scheduler role scales like the others (DEC-020).
- Affected artifacts: `crates/kampus-tasks`, the task queue port.
