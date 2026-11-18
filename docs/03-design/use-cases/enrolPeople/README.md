---
id: UC-016
name: enrolPeople
title: Enrol people in a course
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-016 Enrol people in a course — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/enrolPeople/README.md) | [Analysis](../../../02-analysis/use-cases/enrolPeople/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/enrolPeople/README.md) | [Tests](../../../04-implementation/use-cases/enrolPeople/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/enrolPeople/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Enrolment service | service | `crates/enrol/src/enrolment.rs` |
| `POST /api/v1/enrolment-instances/{instance_id}/enrolments` | API endpoint | `docs/03-design/api/enrolment.openapi.yaml` |
| `enrol.enrolment`, `enrol.enrolment_grant` | tables | `docs/03-design/data-model/enrolment.md` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| enrol.enrolment, enrol.enrolment_grant | insert |
| access.grant | insert (through the access service) |
