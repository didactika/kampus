---
id: UC-036
name: reviewAuditLog
title: Review the audit log
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-036 Review the audit log — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/reviewAuditLog/README.md) | [Analysis](../../../02-analysis/use-cases/reviewAuditLog/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/reviewAuditLog/README.md) | [Tests](../../../04-implementation/use-cases/reviewAuditLog/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/reviewAuditLog/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Audit query service | service | `crates/kampus-events/src/audit.rs` |
| `/api/v1/audit` | API endpoints | `docs/03-design/api/events.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| events.audit_entry | select |
