---
id: DEC-042
status: proposed
---

# DEC-042 Derived enrolments from system groups and courses, and expiry as a recorded job

- Date: 2026-11-18
- Context: Programmes and companion courses share participants; ends of enrolments must be applied reliably. Chained links and guesses about which role to remove create surprises.
- Options considered: copy members once; live queries across courses; derived enrolments kept in sync by a job, no chains, and expiry rules applied by a job with recorded outcomes (chosen).
- Decision:
  - System group and course-link instances derive their enrolments; a sync job adds and removes exactly the enrolments of that instance when the source changes.
  - Linked enrolments are never a source for another link; a link that would create a cycle is refused.
  - When an enrolment ends, a scheduled job applies the instance's expiry rule (suspend, remove, keep without role) and records the outcome.
- Consequences: Programmes stay in sync without manual work; links cannot loop; every expiry is visible and explainable.
- Affected artifacts: `crates/kampus-enrol`, `crates/kampus-users` (system groups), `crates/kampus-tasks`.
