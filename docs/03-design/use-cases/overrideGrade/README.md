---
id: UC-028
name: overrideGrade
title: Override or lock a grade
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-028 Override or lock a grade — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/overrideGrade/README.md) | [Analysis](../../../02-analysis/use-cases/overrideGrade/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/overrideGrade/README.md) | [Tests](../../../04-implementation/use-cases/overrideGrade/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/overrideGrade/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Grade service | service | `crates/records/src/grade.rs` |
| override and lock endpoints | API endpoints | `docs/03-design/api/records.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| records.grade, records.entry | update, insert |
| records.record | update (lock) |
