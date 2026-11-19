---
id: UC-012
name: createCourse
title: Create and configure a course
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-012 Create and configure a course — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/createCourse/README.md) | [Analysis](../../../02-analysis/use-cases/createCourse/README.md) | [Design](../../../03-design/use-cases/createCourse/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/createCourse/README.md#tests) |

Design: [README](../../../03-design/use-cases/createCourse/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Course service | service | `crates/courses/src/course.rs` | no (to build) |
| `/api/v1/courses/categories/{category_id}/courses`, `/api/v1/courses/courses/{course_id}`, state | API endpoints | `docs/03-design/api/courses.openapi.yaml` | no (to build) |
| `courses.course`, `courses.course_owner` | tables | `docs/03-design/data-model/courses.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-041, RN-042, RN-049 to RN-051 | `crates/courses/tests/` | planned |
