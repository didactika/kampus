---
id: UC-021
name: addActivity
title: Add an activity or material
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-021 Add an activity or material — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/addActivity/README.md) | [Analysis](../../../02-analysis/use-cases/addActivity/README.md) | [Design](../../../03-design/use-cases/addActivity/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/addActivity/README.md#tests) |

Design: [README](../../../03-design/use-cases/addActivity/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Course module service | service | `crates/kampus-activities/src/module.rs` | no (to build) |
| `POST /api/v1/sections/{section_id}/course-modules` | API endpoint | `docs/03-design/api/activities.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-075 to RN-078 | `crates/kampus-activities/tests/` | planned |
