---
id: DEC-028
status: proposed
---

# DEC-028 Grants with a validity period, owned by their source

- Date: 2026-11-18
- Context: People hold roles for a term, a replacement or a review; modules (enrolment, system groups) give roles automatically. Removing a role must never remove one that another source gave.
- Options considered: permanent grants removed by hand; one grant per person and role whatever the source; grants with validity and an owner (chosen).
- Decision:
  - A grant has `valid_from` and an optional `valid_until`; expired grants stop counting and stay as history.
  - A grant is manual or owned by a module; a module owns its grants through its own link table (it is higher in the ranking) and is the only one that removes them.
  - Manual grants are unique per role, scope and person; module grants are unique through their link.
- Consequences: Temporary roles end on time without anyone remembering; removing an enrolment never removes a role a person was given by hand; no polymorphic owner column.
- Affected artifacts: `crates/kampus-access`, `crates/kampus-enrol`, `crates/kampus-users`.
