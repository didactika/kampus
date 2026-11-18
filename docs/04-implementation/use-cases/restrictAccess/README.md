---
id: UC-022
name: restrictAccess
title: Restrict access to an activity or section
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-022 Restrict access to an activity or section — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/restrictAccess/README.md) | [Analysis](../../../02-analysis/use-cases/restrictAccess/README.md) | [Design](../../../03-design/use-cases/restrictAccess/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/restrictAccess/README.md#tests) |

Design: [README](../../../03-design/use-cases/restrictAccess/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Condition service and evaluators | service | `crates/kampus-activities/src/conditions.rs` | no (to build) |
| `/api/v1/course-modules/{course_module_id}/conditions` | API endpoint | `docs/03-design/api/activities.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-080 to RN-083 | `crates/kampus-activities/tests/` | planned |
