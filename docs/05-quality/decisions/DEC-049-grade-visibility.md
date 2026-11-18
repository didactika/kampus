---
id: DEC-049
status: proposed
---

# DEC-049 One visibility service for every read of grades

- Date: 2026-11-18
- Context: Hidden grades leak when some consumers (rules, achievements, integrations) read stored totals that include them while reports correct them at display time.
- Options considered: correct at display time in each report; store two totals; apply visibility in one read service used by everyone (chosen).
- Decision:
  - Every read of grades and totals, by people or modules, goes through the records read service with the requester's context.
  - The service returns what the requester may see; a teacher view is explicit and needs a permission.
  - Progress rules, achievements, integrations and exports use the same service.
- Consequences: Hidden grades never leak, whoever asks.
- Affected artifacts: `crates/kampus-records`, `crates/kampus-progress`, achievements, the API.
