---
id: DEC-040
status: proposed
---

# DEC-040 Enrolment is participation; roles come from grants the enrolment owns

- Date: 2026-11-18
- Context: People join courses in several ways at once (by hand, by themselves, through a programme). Mixing participation and permissions makes it impossible to know which role to remove when one way ends, and every module computing "is this person in the course" on its own gives different answers.
- Options considered: roles stored on the enrolment; roles assigned separately without a link; enrolments that own their grants, with one computed active participation (chosen).
- Decision:
  - An enrolment records participation through one method instance; the roles it gives are grants it owns (DEC-028).
  - A person is an active participant when any of their enrolments in the course is active (status, instance enabled, database clock within the period); this is computed once per request and used by every module.
  - Methods are plugins with typed instance data; a validated settings document holds only what a method declares.
  - Other modules react to enrolment events; none reads enrolment tables.
- Consequences: Removing one way of joining never removes a role given by another; one answer to "is this person in the course"; new methods (payments, external systems) plug in without new generic columns.
- Affected artifacts: `crates/kampus-enrol`, `crates/kampus-access`, every module that filters by participation.
