---
id: UC-022
name: restrictAccess
title: Restrict access to an activity or section
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-022 Restrict access to an activity or section — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/restrictAccess/README.md) | [Analysis](../../../02-analysis/use-cases/restrictAccess/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/restrictAccess/README.md) | [Tests](../../../04-implementation/use-cases/restrictAccess/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/restrictAccess/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Condition service and evaluators | service | `crates/activities/src/conditions.rs` |
| `/api/v1/course-modules/{course_module_id}/conditions` | API endpoint | `docs/03-design/api/activities.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| activities.condition and its leaf tables | replace |
