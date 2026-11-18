---
id: DEC-023
status: proposed
---

# DEC-023 Exact numerics for grades, weights and money

- Date: 2026-11-18
- Context: Grades, weights and fees are added, averaged and compared; floating point makes totals differ by a hundredth between screens.
- Options considered: floating point; integers in minor units; numeric with declared precision (chosen).
- Decision:
  - Grades numeric(12,5), percentages and weights numeric(9,5), money numeric(19,4) with an ISO 4217 currency.
  - Rounding is explicit, named and applied only at the boundaries that need it.
  - Flags are booleans; enumerations are checked values or lookup tables.
- Consequences: Totals are exact and reproducible; the API sends exact numbers as strings; aggregation rules can be tested against worked examples.
- Affected artifacts: `crates/kampus-common`, `kampus-records`, payments, the API.
