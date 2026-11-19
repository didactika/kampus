---
id: UC-015
name: chooseCourseLayout
title: Choose a course layout
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-015 Choose a course layout — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/chooseCourseLayout/README.md) | [Analysis](../../../02-analysis/use-cases/chooseCourseLayout/README.md) | [Design](../../../03-design/use-cases/chooseCourseLayout/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/chooseCourseLayout/README.md#tests) |

Design: [README](../../../03-design/use-cases/chooseCourseLayout/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Layout registry | service | `crates/courses/src/layout.rs` | no (to build) |
| `PUT /api/v1/courses/courses/{course_id}/layout` | API endpoint | `docs/03-design/api/courses.openapi.yaml` | no (to build) |
| `courses.course_layout_setting` | table | `docs/03-design/data-model/courses.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-052 | `crates/courses/tests/` | planned |
