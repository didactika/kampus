---
id: DEC-046
status: accepted
---

# DEC-046 One exception model for every activity, with one precedence rule

- Date: 2026-11-18
- Context: People need extra time, other dates or more attempts in any activity, not only in a few types, and every place that shows a date must agree on the effective value (RF-054).
- Options considered: overrides per activity type; a generic key-value table; one typed exception table and one resolver (chosen).
- Decision:
  - Exceptions have typed columns (opens, closes, due, cut-off, time limit, attempts) and target a person xor a group, unique per course module and target.
  - Precedence: the person's exception wins setting by setting; among groups, the most generous value; otherwise the activity's value.
  - One resolver is used by activities, the calendar, the course page, notifications and the API; group deletion removes its exceptions by cascade.
- Consequences: Extensions and special arrangements in any activity, with the same answer everywhere.
- Affected artifacts: `crates/kampus-activities`, `crates/kampus-calendar`, every activity type with dates.
