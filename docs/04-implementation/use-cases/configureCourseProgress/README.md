---
id: UC-031
name: configureCourseProgress
title: Configure course progress
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-031 Configure course progress — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/configureCourseProgress/README.md) | [Analysis](../../../02-analysis/use-cases/configureCourseProgress/README.md) | [Design](../../../03-design/use-cases/configureCourseProgress/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/configureCourseProgress/README.md#tests) |

Design: [README](../../../03-design/use-cases/configureCourseProgress/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Course progress service | service | `crates/kampus-progress/src/course.rs` | no (to build) |
| criteria and prerequisites endpoints | API endpoints | `docs/03-design/api/progress.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-120 to RN-122 | `crates/kampus-progress/tests/` | planned |
