---
id: DEC-038
status: proposed
---

# DEC-038 Groups per course or section, group mode as a scoped setting, owned memberships

- Date: 2026-11-18
- Context: Groups split participants for work and visibility. Teams often belong to one part of a course, and enrolment methods add people to groups automatically.
- Options considered: one group mode per course; group mode per activity only; group mode as a setting resolved course, section, activity, with groups optionally limited to a section (chosen).
- Decision:
  - A group belongs to a course and may be limited to one section; members must be enrolled and able to reach that section.
  - Group mode (none, separate, visible) is a setting resolved through the scopes course, section, activity.
  - Memberships are manual or owned by a module (enrolment, a rule), and only their origin removes them.
  - Group names and codes are unique in the course; group sets contain each group once; joining keys are hashed.
- Consequences: One section can work in teams while the rest of the course does not; automatic memberships never remove manual ones.
- Affected artifacts: `crates/kampus-courses`, `crates/kampus-enrol`, `crates/kampus-settings`.
