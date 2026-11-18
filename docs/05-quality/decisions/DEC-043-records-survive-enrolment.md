---
id: DEC-043
status: accepted
---

# DEC-043 Ending a participation never deletes academic records or progress

- Date: 2026-11-18
- Context: People leave a course and come back: a suspension, a change of group, a mistake corrected later. Deleting grades and progress when the last enrolment ends, and relying on a history that may have been purged to restore them, loses work.
- Options considered: delete with the enrolment and restore from history; keep and show everything; keep, hide while not participating, and show again on return (chosen).
- Decision:
  - Academic records and progress belong to the person in the course, not to the enrolment.
  - While the person is not an active participant, their records and progress are hidden from course views and reports, and shown again when they return.
  - They are deleted only by course deletion, retention rules or a privacy request.
- Consequences: Re-enrolling never loses work; teachers can still see former participants where a permission allows; retention policies decide what is kept and for how long.
- Affected artifacts: `crates/records`, `crates/progress`, `crates/enrol`, retention settings.
