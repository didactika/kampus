---
id: UC-018
name: manageEnrolment
title: Manage an enrolment
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-018 Manage an enrolment — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageEnrolment/README.md) | [Analysis](../../../02-analysis/use-cases/manageEnrolment/README.md) | [Design](../../../03-design/use-cases/manageEnrolment/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/manageEnrolment/README.md#tests) |

Design: [README](../../../03-design/use-cases/manageEnrolment/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Enrolment service | service | `crates/kampus-enrol/src/enrolment.rs` | no (to build) |
| `/api/v1/enrolments/{enrolment_id}` | API endpoint | `docs/03-design/api/enrolment.openapi.yaml` | no (to build) |
| `enrol.enrolment` | table | `docs/03-design/data-model/enrolment.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-060, RN-061, RN-069, RN-070 | `crates/kampus-enrol/tests/` | planned |
