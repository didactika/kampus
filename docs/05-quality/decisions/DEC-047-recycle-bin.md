---
id: DEC-047
status: proposed
---

# DEC-047 Deletion through a recycle bin with a visible failed state

- Date: 2026-11-18
- Context: Deleting an activity in the background can fail and leave content hidden from everyone with no way to recover it.
- Options considered: immediate deletion; background deletion without a trace; a recycle bin with retention, restore and a visible failed state (chosen).
- Decision:
  - Deleting a course module or a section moves it to the course recycle bin in one transaction.
  - Owned data is deleted by cascade when the bin item expires (30 days by default); restoring puts it back in its original place when it still exists.
  - A purge that fails is a visible state of the bin item with its reason; teachers can retry or restore it.
- Consequences: Mistakes can be undone; failures are never invisible.
- Affected artifacts: `crates/kampus-activities`, `crates/kampus-courses`, the course editor.
