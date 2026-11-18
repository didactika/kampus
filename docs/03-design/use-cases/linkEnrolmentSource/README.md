---
id: UC-019
name: linkEnrolmentSource
title: Link a system group or a course
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-019 Link a system group or a course — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/linkEnrolmentSource/README.md) | [Analysis](../../../02-analysis/use-cases/linkEnrolmentSource/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/linkEnrolmentSource/README.md) | [Tests](../../../04-implementation/use-cases/linkEnrolmentSource/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/linkEnrolmentSource/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Sync service | service | `crates/kampus-enrol/src/sync.rs` |
| `PUT /api/v1/enrolment-instances/{instance_id}/source` | API endpoint | `docs/03-design/api/enrolment.openapi.yaml` |
| `enrol.linked_group`, `enrol.linked_course` | tables | `docs/03-design/data-model/enrolment.md` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| enrol.linked_group, enrol.linked_course | insert |
| enrol.enrolment | insert, delete (this instance only) |
