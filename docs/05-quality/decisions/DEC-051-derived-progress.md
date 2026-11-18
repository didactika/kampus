---
id: DEC-051
status: accepted
---

# DEC-051 Progress as derived records re-evaluated by events

- Date: 2026-11-18
- Context: Progress must follow views, submissions, grades and exceptions; recalculations must never erase what people decided; completion must be revocable when its criteria stop being met.
- Options considered: progress written once; periodic full recalculation; derived records re-evaluated by listeners, with people's decisions kept (chosen).
- Decision:
  - Activity and course progress are re-evaluated by listeners of the relevant events, with history and an event per change.
  - Manual ticks and progress exceptions are records of people: kept, visible, never overwritten.
  - States are an enumeration; how a state is shown is decided at read time through the grade visibility service.
  - Course completion is revoked when a criterion stops being met, unless the course makes it final; both are recorded.
- Consequences: Progress is always current and explainable; nothing a person decided is lost.
- Affected artifacts: `crates/kampus-progress`, `crates/kampus-records`.
