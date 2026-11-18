---
id: UC-024
name: copyActivity
title: Copy or move an activity
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-024 Copy or move an activity — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/copyActivity/README.md) | [Analysis](../../../02-analysis/use-cases/copyActivity/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/copyActivity/README.md) | [Tests](../../../04-implementation/use-cases/copyActivity/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/copyActivity/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Course module service | service | `crates/kampus-activities/src/module.rs` |
| `/api/v1/course-modules/{course_module_id}/copy`, `/move` | API endpoints | `docs/03-design/api/activities.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| activities.course_module and the type schema | insert or update |
| access.permission_override | insert (through the access service) |
