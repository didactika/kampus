---
id: DEC-065
status: accepted
---

# DEC-065 Database port: neutral schema, portable types, a typed query builder and migrations tested per dialect

- Date: 2026-11-18
- Context: DEC-009 requires several relational databases. Schemas written per dialect drift; dialect-specific types leak into modules; migrations tested only on one database fail on the others.
- Options considered: hand-written SQL per dialect; a full ORM; a neutral schema with a typed builder and capabilities (chosen).
- Decision:
  - Each module writes its schema once in a neutral format with a closed list of portable types; the port generates migrations per dialect with the installation prefix (DEC-039).
  - Queries use a typed, dialect-neutral builder; raw SQL lives only in dialect adapters, with a written reason, covered by the suite.
  - Row locks with `SKIP LOCKED`, upserts and advisory locks are capabilities with documented fallbacks.
  - Migrations are forward-only and tested from the previous release on every supported dialect.
  - The schema linter of DEC-014 runs on the neutral schema: foreign keys, types, natural keys, indexes.
- Consequences: One schema per module, checked once and run everywhere; adding a database is an adapter plus a green suite.
- Affected artifacts: `crates/kampus-db`, `crates/kampus-db-postgres`, every module's schema, RNF-016, RNF-021.
