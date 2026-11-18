---
id: DEC-059
status: accepted
---

# DEC-059 Declared, typed settings resolved by scope, with locks, fixed values and history

- Date: 2026-11-18
- Context: Settings stored as untyped name and value rows are validated (or not) by each screen, cannot say where a value came from, and keep no record of who changed them; secrets in them are readable by anyone with access to the page.
- Options considered: one key and value table; settings per module in their own tables; declared typed settings in one module, resolved through the scope hierarchy of DEC-008 (chosen).
- Decision:
  - Modules declare each setting with a JSON schema, a default, the levels where it may be set and its permission.
  - Values are stored per scope and resolved from the most specific level up; the answer carries its origin.
  - Locks stop overrides below a level; deployment configuration can fix a value, which is then read-only.
  - Sensitive values are encrypted and never returned; every change is recorded and published for cache invalidation.
- Consequences: Every value is valid and explainable; the customisation framework (DEC-008) builds on the same resolution.
- Affected artifacts: `crates/settings`, every module's declarations.
