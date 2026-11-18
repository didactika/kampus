---
id: UC-031
name: configureCourseProgress
title: Configure course progress
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-031 Configure course progress — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/configureCourseProgress/README.md) | [Analysis](../../../02-analysis/use-cases/configureCourseProgress/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/configureCourseProgress/README.md) | [Tests](../../../04-implementation/use-cases/configureCourseProgress/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/configureCourseProgress/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Course progress service | service | `crates/progress/src/course.rs` |
| criteria and prerequisites endpoints | API endpoints | `docs/03-design/api/progress.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| progress.criterion and its typed tables, progress.prerequisite | replace |
| progress.course_state | update by the job |
