---
id: UC-019
name: linkEnrolmentSource
title: Link a system group or a course
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-019 Link a system group or a course — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/linkEnrolmentSource/README.md) | [Analysis](../../../02-analysis/use-cases/linkEnrolmentSource/README.md) | [Design](../../../03-design/use-cases/linkEnrolmentSource/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/linkEnrolmentSource/README.md#tests) |

Design: [README](../../../03-design/use-cases/linkEnrolmentSource/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Sync service | service | `crates/kampus-enrol/src/sync.rs` | no (to build) |
| `PUT /api/v1/enrolment-instances/{instance_id}/source` | API endpoint | `docs/03-design/api/enrolment.openapi.yaml` | no (to build) |
| `enrol.linked_group`, `enrol.linked_course` | tables | `docs/03-design/data-model/enrolment.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-067, RN-073 | `crates/kampus-enrol/tests/` | planned |
