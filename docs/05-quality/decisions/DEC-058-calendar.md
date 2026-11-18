---
id: DEC-058
status: accepted
---

# DEC-058 The calendar as a projection, recurrences expanded on read and subscriptions diffed by UID

- Date: 2026-11-18
- Context: Modules that copy their dates into calendar rows drift when dates or exceptions change; storing one row per occurrence makes series hard to edit.
- Options considered: modules write calendar rows; one row per occurrence; a projection of activity dates refreshed by events and one row per series (chosen).
- Decision:
  - Activity dates are a projection refreshed by events; exceptions (DEC-046) apply when read.
  - Entries have one scope; recurring entries store an RFC 5545 rule, expanded for the requested window, with per-occurrence exceptions.
  - Subscriptions are refreshed by a job that matches entries by UID and sequence and records every change.
  - Personal feeds use scoped, revocable tokens stored hashed.
- Consequences: Dates are the same in the activity, the calendar and notifications; series are edited once.
- Affected artifacts: `crates/kampus-calendar`.
