---
id: UC-024
name: copyActivity
title: Copy or move an activity
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-024 Copy or move an activity — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/copyActivity/README.md) | [Analysis](../../../02-analysis/use-cases/copyActivity/README.md) | [Design](../../../03-design/use-cases/copyActivity/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/copyActivity/README.md#tests) |

Design: [README](../../../03-design/use-cases/copyActivity/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Course module service | service | `crates/kampus-activities/src/module.rs` | no (to build) |
| `/api/v1/course-modules/{course_module_id}/copy`, `/move` | API endpoints | `docs/03-design/api/activities.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-076, RN-077, RN-090 | `crates/kampus-activities/tests/` | planned |
