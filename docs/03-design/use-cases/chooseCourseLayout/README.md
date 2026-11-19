---
id: UC-015
name: chooseCourseLayout
title: Choose a course layout
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-015 Choose a course layout — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/chooseCourseLayout/README.md) | [Analysis](../../../02-analysis/use-cases/chooseCourseLayout/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/chooseCourseLayout/README.md) | [Tests](../../../04-implementation/use-cases/chooseCourseLayout/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/chooseCourseLayout/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Layout registry | service | `crates/courses/src/layout.rs` |
| `PUT /api/v1/courses/courses/{course_id}/layout` | API endpoint | `docs/03-design/api/courses.openapi.yaml` |
| `courses.course_layout_setting` | table | `docs/03-design/data-model/courses.md` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| courses.course, courses.course_layout_setting | update, upsert |
