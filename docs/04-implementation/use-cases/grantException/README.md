---
id: UC-023
name: grantException
title: Grant an exception
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-023 Grant an exception — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/grantException/README.md) | [Analysis](../../../02-analysis/use-cases/grantException/README.md) | [Design](../../../03-design/use-cases/grantException/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/grantException/README.md#tests) |

Design: [README](../../../03-design/use-cases/grantException/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Exception resolver | service | `crates/activities/src/exceptions.rs` | no (to build) |
| `/api/v1/activities/course-modules/{course_module_id}/exceptions`, effective values | API endpoints | `docs/03-design/api/activities.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-084 to RN-087 | `crates/activities/tests/` | planned |
