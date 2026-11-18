---
id: DEC-052
status: proposed
---

# DEC-052 Typed course criteria, an acyclic prerequisite graph and one tracked population

- Date: 2026-11-18
- Context: Course completion combines activities, grades, dates, durations, approvals and other courses; generic criteria tables with free columns hide errors, and cyclic prerequisites block learners forever.
- Options considered: one generic table; free expressions; one typed table per criterion kind (chosen).
- Decision:
  - One table per criterion kind with foreign keys to its targets; one evaluation function per kind.
  - Prerequisites form a graph validated on save; cycles are refused and the graph is shown to the teacher.
  - The tracked population is the same for every rule: active participants with `progress.tracked`.
- Consequences: Criteria that cannot be met are impossible to configure; every rule counts the same people.
- Affected artifacts: `crates/kampus-progress`.
