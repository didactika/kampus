---
id: UC-017
name: selfEnrol
title: Enrol oneself in a course
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-017 Enrol oneself in a course — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/selfEnrol/README.md) | [Analysis](../../../02-analysis/use-cases/selfEnrol/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/selfEnrol/README.md) | [Tests](../../../04-implementation/use-cases/selfEnrol/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/selfEnrol/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Self-enrolment service | service | `crates/enrol/src/self_enrol.rs` |
| `POST /api/v1/enrol/enrolment-instances/{instance_id}/self` | API endpoint | `docs/03-design/api/enrolment.openapi.yaml` |
| `enrol.self_key`, `enrol.request`, `enrol.self_attempt` | tables | `docs/03-design/data-model/enrolment.md` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| enrol.enrolment, enrol.request, enrol.self_attempt | insert |
| access.grant, courses.group_member | insert (through their services) |
