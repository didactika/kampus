---
id: DEC-019
status: proposed
---

# DEC-019 Observability with OpenTelemetry, and an audit log apart

- Date: 2026-11-18
- Context: operators need to see requests, jobs and failures across stateless instances (RNF-009); administrators need to know who did what, which is a different question with different retention.
- Options considered: plain text logs; a vendor agent; OpenTelemetry traces, metrics and structured logs, plus an audit log written by the event system (chosen).
- Decision:
  - Traces and metrics through OpenTelemetry (`tracing` in Rust), exported to a collector; structured JSON logs with the request id and trace id.
  - Health and readiness endpoints per role; metrics for requests, jobs (queued, running, failed), outbox lag, cache hit rates and database pools.
  - The **audit log** is append-only, written from domain events, with its own retention policy per campus, and never mixed with operational logs.
- Consequences: any backend that speaks OpenTelemetry works; the reference deployment ships dashboards and alerts; audit queries never scan operational logs.
- Affected artifacts: `crates/kampus-server`, `crates/kampus-events`, the operator guide, the deployment.
