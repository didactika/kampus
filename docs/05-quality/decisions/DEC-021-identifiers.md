---
id: DEC-021
status: proposed
---

# DEC-021 UUID version 7 keys, natural keys in their scope, typed ids

- Date: 2026-11-18
- Context: Records are imported, exported, merged between campuses and tenants, referenced by external systems and shown in URLs. Sequential integers leak counts, can be enumerated, and collide when data moves.
- Options considered: sequential integers; random UUIDs (version 4); UUID version 7 with natural keys beside them (chosen).
- Decision:
  - Every row has a UUID version 7 primary key: globally unique, ordered by creation time (good index locality), safe to expose.
  - Things people type or read also have a natural key, unique in its scope and enforced by a constraint; references always use the UUID.
  - External identifiers live in external references (system, id), never in the record's own columns.
  - Rust uses one identifier type per entity.
- Consequences: No enumeration through URLs, no collisions when importing or merging, and identifiers that cannot be mixed up at compile time; UUID columns are larger than integers, which the time ordering of version 7 keeps cheap to index.
- Affected artifacts: `crates/kampus-common`, every schema, the API, the importer.
