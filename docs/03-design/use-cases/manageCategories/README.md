---
id: UC-011
name: manageCategories
title: Manage course categories
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-011 Manage course categories — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageCategories/README.md) | [Analysis](../../../02-analysis/use-cases/manageCategories/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/manageCategories/README.md) | [Tests](../../../04-implementation/use-cases/manageCategories/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/manageCategories/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Category service | service | `crates/kampus-courses/src/category.rs` |
| `/api/v1/campuses/{campus_id}/categories`, `/api/v1/categories/{category_id}`, move | API endpoints | `docs/03-design/api/courses.openapi.yaml` |
| `courses.category` | table | `docs/03-design/data-model/courses.md` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| courses.category | insert, update, delete |
| access.scope, access.scope_closure | insert, update |
