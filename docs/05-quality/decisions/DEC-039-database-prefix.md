---
id: DEC-039
status: accepted
---

# DEC-039 An installation prefix on every physical database name

- Date: 2026-11-18
- Context: table names such as `group`, `grant`, `order` or `user` are reserved words in SQL dialects; several installations (test, staging, a second campus) sometimes have to share one database; and the database port must map one schema description to dialects with and without schemas (DEC-009).
- Options considered: quote every identifier; rename tables to avoid every reserved word of every dialect; a configurable installation prefix on every physical name, with short logical names in the specification and the code (chosen).
- Decision:
  - Every physical namespace the platform creates starts with the **installation prefix**, `kmp_` by default: on PostgreSQL the schemas (one per module), on dialects without schemas the tables themselves.
  - The prefix is set at installation with `KAMPUS_DATABASE_PREFIX` (lower-case letters and digits, starting with a letter, 2 to 8 characters, ending with `_`) and cannot change afterwards; the platform refuses to start if it differs from the one recorded in the database.
  - The specification and the code use **logical names**: `module.table` (for example `courses.group`). The database port maps them: on PostgreSQL to schema `{prefix}{module}` and table `{table}` (`kmp_courses.group`), since the prefixed schema already separates installations; on dialects without schemas to `{prefix}{module}_{table}` (`kmp_courses_group`).
  - The port always quotes identifiers in the SQL it generates, so a table may keep a natural name that is a reserved word (`group`, `grant`, `order`).
  - Plugins get the same treatment, with their own module name; they never write physical names.
- Consequences: natural table names without reserved-word conflicts in any dialect; several installations can share one database or one PostgreSQL cluster; every SQL statement goes through the port's name mapping, which the schema linter (DEC-014) checks.
- Affected artifacts: `crates/kampus-db`, every module's migrations, the plugin SDK, the operator guide, DEC-014.
