---
id: DEC-048
status: accepted
---

# DEC-048 Grades as an immutable ledger, totals derived by a job

- Date: 2026-11-18
- Context: Grades change from activities, teachers, imports and formulas; totals depend on many grades. Recalculating on read makes reads slow and racy; incomplete history makes disputes impossible to settle.
- Options considered: mutable grades with partial history; recalculation on read; an append-only ledger with current grades upserted and totals derived by a job (chosen).
- Decision:
  - Every change is an immutable entry (raw, feedback, source, actor, time).
  - The current grade per record and person is an idempotent upsert; the same grade twice changes nothing and publishes one event.
  - Totals and formulas are derived by a job along the dependency graph, only for what changed; reads never recalculate.
- Consequences: Full history for every grade; fast, side-effect-free reads; concurrent first grades never fail.
- Affected artifacts: `crates/records`, every graded activity type.
