---
id: UC-016
name: enrolPeople
title: Enrol people in a course
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-016 Enrol people in a course — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/enrolPeople/README.md) | [Analysis](../../../02-analysis/use-cases/enrolPeople/README.md) | [Design](../../../03-design/use-cases/enrolPeople/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/enrolPeople/README.md#tests) |

Design: [README](../../../03-design/use-cases/enrolPeople/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Enrolment service | service | `crates/enrol/src/enrolment.rs` | no (to build) |
| `POST /api/v1/enrolment-instances/{instance_id}/enrolments` | API endpoint | `docs/03-design/api/enrolment.openapi.yaml` | no (to build) |
| `enrol.enrolment`, `enrol.enrolment_grant` | tables | `docs/03-design/data-model/enrolment.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-061 to RN-063, RN-073 | `crates/enrol/tests/` | planned |
