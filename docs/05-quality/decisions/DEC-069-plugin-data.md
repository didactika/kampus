---
id: DEC-069
status: accepted
---

# DEC-069 Plugin data in its own schema, migrated by the host from a neutral schema

- Date: 2026-11-18
- Context: Plugins that create and alter tables freely, or write to platform tables, break integrity and upgrades.
- Options considered: plugins share the platform tables; key and value storage only; one schema per plugin with the neutral schema of DEC-065 (chosen).
- Decision:
  - A plugin declares its tables in the neutral schema format; the host creates them in a schema per plugin with the installation prefix.
  - Foreign keys to platform tables are allowed only to the identifiers the read models expose (accounts, courses, course modules, scopes), with `CASCADE` or `RESTRICT` declared.
  - Migrations are forward-only, one transaction per step, with checksums.
  - The plugin reaches its tables only through the `storage` interface with structured filters.
- Consequences: Plugin data is relational and portable like the core's, and removing a plugin removes exactly its schema.
- Affected artifacts: `crates/plugin-host`, `crates/db`.
