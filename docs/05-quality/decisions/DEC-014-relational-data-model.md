---
id: DEC-014
status: accepted
---

# DEC-014 A truly relational, optimised data model

- Date: 2026-11-17
- Context: The owner requires the database of Kampus to be truly relational and optimised: a structure that is solid and well organised. Long-lived schemas commonly fall into these anti-patterns, which Kampus rules out from the start:
  - foreign keys declared in a schema description but never created in the database;
  - integer columns that act as references without a constraint;
  - polymorphic references (an item id beside a component, item type or scope column);
  - times stored as integers;
  - integer flags instead of booleans;
  - structured data stored in text fields;
  - tables with no unique constraint besides the primary key.
- Options considered: a schema without database-enforced integrity; let the database port (DEC-009) hide integrity in code; design a relational model with the integrity in the database (chosen).
- Decision: the database enforces the data model. Every module follows these rules, checked in CI:
  1. **Real foreign keys.** Every reference is a `FOREIGN KEY` constraint with an explicit `ON DELETE` (`RESTRICT` by default, `CASCADE` only for owned children, `SET NULL` only for optional links). No orphan rows.
  2. **No polymorphic references.** A reference points to one table. Where a concept can attach to several kinds of things, use one of these instead:
     - a typed association table per kind;
     - a supertype table that every kind references, for example the scope table (DEC-007);
     - an exclusive arc: one nullable foreign key per kind, with a `CHECK` that exactly one is set.
  3. **Hierarchies as data.** The scope hierarchy (campus → category → course → section → course module → widget) and nested categories and sections use parent foreign keys plus a closure table maintained in transactions. No materialised path strings.
  4. **Proper types.** Timestamps with time zone, booleans, exact numerics for grades and money, and enumerations as lookup tables or checked values. JSON (`jsonb`) only for data that is really schemaless, such as plugin settings validated by their schema, never for structured data the platform queries.
  5. **Constraints by default.** Columns are `NOT NULL` unless absence has a meaning. Natural keys have `UNIQUE` constraints, such as the shortname of a course in its campus, or a user in a course. Invariants are `CHECK` constraints, such as an end date after its start date.
  6. **Normalised to third normal form.** Denormalisation only where a measurement justifies it. It is documented and kept correct by the database (generated columns, materialised views, triggers), never by application code alone.
  7. **Modular ownership.** One PostgreSQL schema per module (`access`, `courses`, `records`…), matching the crates (DEC-002). A module writes only its own schema. Cross-module foreign keys point only to modules lower in the dependency ranking.
  8. **Indexes from queries.** Every foreign key is indexed. Other indexes come from the queries of the use cases. Query plans of key use cases are checked in tests, and no sequential scan on large tables is accepted.
  9. **Identifiers and auditing.** Surrogate keys are chosen once, in the platform foundations. Every table has `created_at` and `updated_at`, plus `created_by` where an actor creates it. History goes to history tables, not to soft-delete flags, except where retention rules (privacy) require it.
  10. **Migrations.** Versioned and forward-only, following the expand-and-contract pattern for zero-downtime upgrades (RNF-006, DEC-080).
  11. **Portability.** Foreign keys, unique constraints, check constraints and proper types exist in every database of DEC-009. Where a dialect lacks a type, the database port maps it, and the conformance suite tests the constraint.
  12. **Generated data dictionary.** Each module's tables, columns, constraints and relations are documented from the schema itself, in the handbook (DEC-011).
- Consequences:
  - The design of each module lists the anti-patterns it avoids, and its data model is checked against these rules before any code.
  - A schema linter runs in CI. It rejects reference columns without a foreign key, integer timestamps, integer flags, missing indexes on foreign keys, and tables without a natural key.
  - The importer (RF-111) must clean data that the source platform allowed to become inconsistent (orphans, duplicates). It reports what it repaired.
- Affected artifacts: RNF-021, the data model conventions, the definition of done, `crates/db`.
- Refined by: DEC-065 (database port), DEC-079 (ownership, supertypes and checked dependencies) and DEC-080 (migrations); rules in the [data model conventions](../../03-design/data-model/conventions.md).
