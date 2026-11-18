---
id: DEC-034
status: proposed
---

# DEC-034 Visibility stored per node, computed through the scope tree

- Date: 2026-11-18
- Context: Hiding a category, a course or a section must hide everything below it for learners, and showing it again must restore exactly what each part had. Copying the state down to every child, with a one-level undo, loses the original state when hide and show operations nest.
- Options considered: copy the state down with an undo column; compute visibility on every read without cache; store each node's own setting and compute the effective visibility through the scope closure with a cache (chosen).
- Decision:
  - Every category, course, section and course module stores only its own visibility: shown, hidden, or scheduled with a window.
  - The effective visibility is visible only if the node and every ancestor are visible now; it is computed with the scope closure and cached per subtree, invalidated by events.
  - People with `courses.hidden.view` in a scope see hidden content there, marked as hidden.
- Consequences: Nested hide and show never lose anything; one click writes one row; schedules (visible from, until) work at every level.
- Affected artifacts: `crates/kampus-courses`, `crates/kampus-activities`, the course editor.
