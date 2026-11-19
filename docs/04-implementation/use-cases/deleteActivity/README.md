---
id: UC-025
name: deleteActivity
title: Delete and restore an activity
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-025 Delete and restore an activity — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/deleteActivity/README.md) | [Analysis](../../../02-analysis/use-cases/deleteActivity/README.md) | [Design](../../../03-design/use-cases/deleteActivity/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/deleteActivity/README.md#tests) |

Design: [README](../../../03-design/use-cases/deleteActivity/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Bin service and purge job | service | `crates/activities/src/bin.rs` | no (to build) |
| `/api/v1/activities/courses/{course_id}/bin`, restore, retry | API endpoints | `docs/03-design/api/activities.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-088, RN-089 | `crates/activities/tests/` | planned |
