---
id: DEC-080
status: accepted
---

# DEC-080 Forward-only migrations with expand and contract

- Date: 2026-11-18
- Context: DEC-014 asked for reversible migrations and DEC-065 for forward-only ones. Reversing a migration in production loses data written after it, and zero-downtime upgrades (RNF-006) need two releases to run on the same schema.
- Options considered: reversible migrations with down scripts; forward-only migrations with expand and contract (chosen).
- Decision:
  - Migrations are forward-only, one transaction per step, recorded with a checksum.
  - A release only expands (adds tables, columns, nullable or not-yet-validated constraints) and writes both shapes; a later release contracts (removes the old shape) once no supported release reads it.
  - Rolling back a release means running the previous release on the expanded schema.
  - The linter refuses a migration that drops or renames something the previous release reads.
- Consequences: Rolling upgrades and rollbacks without data loss; DEC-014 rule 10 reads "versioned and forward-only, following expand and contract".
- Affected artifacts: `crates/kampus-db`, every module's migrations, the operator guide.
