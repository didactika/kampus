---
id: DEC-045
status: proposed
---

# DEC-045 Access conditions as typed rows with foreign keys

- Date: 2026-11-18
- Context: Conditions depend on other activities, grades, groups and profile fields. Stored as JSON with ids inside, they silently break when their target disappears, and unknown condition types open content.
- Options considered: JSON documents; a rules engine with free expressions; typed rows in a tree, one table per kind, with foreign keys (chosen).
- Decision:
  - A condition tree of all, any and not; each leaf is a row of its kind with a foreign key to its target.
  - Deleting a target is refused while a condition uses it, unless the condition is removed or rewritten in the same operation.
  - A condition of an unknown or disabled kind closes the content and is reported; plugins add kinds with their own tables.
- Consequences: Conditions never break silently; teachers see what depends on what.
- Affected artifacts: `crates/kampus-activities`, condition plugins.
