---
id: DEC-033
status: accepted
---

# DEC-033 Account lifecycle as a state machine; deletion erases and keeps a tombstone

- Date: 2026-11-18
- Context: People join, pause and leave. Deletions must erase personal data (RNF-002) while academic records stay consistent, and every change must be traceable.
- Options considered: free status flags; soft deletion keeping the data; hard deletion breaking references; a state machine with erasure and a tombstone row (chosen).
- Decision:
  - States invited, pending confirmation, active, suspended, deleted; one permission, one reason, one history row and one event per transition.
  - Deletion is scheduled with a grace period (14 days by default) during which the account is suspended and the deletion can be cancelled.
  - When it runs, every module erases its personal data through its privacy handler; the account row stays with its id, its personal columns erased and a pseudonym.
  - Bulk operations are jobs that validate every row first and report per row.
- Consequences: The right to erasure is real, records keep valid references, mistakes can be undone during the grace period, and audits can follow every transition.
- Affected artifacts: `crates/kampus-identity`, `crates/kampus-privacy`, every module's privacy handler.
