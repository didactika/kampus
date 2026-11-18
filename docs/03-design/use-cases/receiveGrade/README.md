---
id: UC-026
name: receiveGrade
title: Receive a grade from an activity
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-026 Receive a grade from an activity — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/receiveGrade/README.md) | [Analysis](../../../02-analysis/use-cases/receiveGrade/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/receiveGrade/README.md) | [Tests](../../../04-implementation/use-cases/receiveGrade/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/receiveGrade/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Grade service | service | `crates/kampus-records/src/grade.rs` |
| Recalculation job | job | `crates/kampus-records/src/jobs/recalculate.rs` |
| `PUT /api/v1/records/{record_id}/grades` | API endpoint | `docs/03-design/api/records.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| records.grade, records.entry | upsert, insert |
| records.recalculation | insert |
