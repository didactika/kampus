---
id: DEC-050
status: accepted
---

# DEC-050 Typed aggregation methods with worked examples as tests

- Date: 2026-11-18
- Context: Weights, extra credit and empty grades mean different things in each method; a single coefficient whose meaning changes with the parent's method confuses teachers and code.
- Options considered: one coefficient column; free formulas only; typed attributes per method with documented examples (chosen).
- Decision:
  - Eight methods (mean, weighted mean, simple weighted mean, natural, median, lowest, highest, mode), each with a typed definition of weights, extra credit, empty grades, drop lowest and keep highest.
  - Each method ships worked examples that are run as tests.
  - Calculation behaviour is versioned per course; changing it is an explicit, recorded action.
- Consequences: Teachers can predict every total; upgrades never change grades silently.
- Affected artifacts: `crates/records`, the handbook.
