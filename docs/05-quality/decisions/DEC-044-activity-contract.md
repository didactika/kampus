---
id: DEC-044
status: accepted
---

# DEC-044 A typed, versioned activity contract

- Date: 2026-11-18
- Context: Activities and materials are plugins (DEC-003). Global functions found by name, with only some operations in a transaction, make every type behave a little differently and break silently when the platform changes.
- Options considered: functions found by naming conventions; a base class to extend; a typed, versioned WIT interface with declared features (chosen).
- Decision:
  - Every activity type implements the activity contract: create, update, copy, reset, delete and export its instance data; declare features, settings schema, permissions, events, grade items and progress rules; ship a UI module.
  - The platform calls the contract inside its own transaction; the type owns a schema whose rows reference `course_module` with cascade.
  - The contract is versioned; a type declares the version it implements, and the host keeps older versions working within a major version (RNF-011).
- Consequences: One behaviour for all types; failures never leave half-created activities; plugins survive upgrades.
- Affected artifacts: `crates/kampus-activities`, the plugin SDK, every activity type.
