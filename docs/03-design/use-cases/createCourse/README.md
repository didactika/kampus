---
id: UC-012
name: createCourse
title: Create and configure a course
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-012 Create and configure a course — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/createCourse/README.md) | [Analysis](../../../02-analysis/use-cases/createCourse/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/createCourse/README.md) | [Tests](../../../04-implementation/use-cases/createCourse/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/createCourse/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Course service | service | `crates/kampus-courses/src/course.rs` |
| `/api/v1/categories/{category_id}/courses`, `/api/v1/courses/{course_id}`, state | API endpoints | `docs/03-design/api/courses.openapi.yaml` |
| `courses.course`, `courses.course_owner` | tables | `docs/03-design/data-model/courses.md` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| courses.course, courses.course_owner, courses.section | insert |
| access.scope, access.grant | insert (through the access service) |
