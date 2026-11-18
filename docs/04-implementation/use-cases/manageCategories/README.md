---
id: UC-011
name: manageCategories
title: Manage course categories
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-011 Manage course categories — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageCategories/README.md) | [Analysis](../../../02-analysis/use-cases/manageCategories/README.md) | [Design](../../../03-design/use-cases/manageCategories/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/manageCategories/README.md#tests) |

Design: [README](../../../03-design/use-cases/manageCategories/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Category service | service | `crates/kampus-courses/src/category.rs` | no (to build) |
| `/api/v1/campuses/{campus_id}/categories`, `/api/v1/categories/{category_id}`, move | API endpoints | `docs/03-design/api/courses.openapi.yaml` | no (to build) |
| `courses.category` | table | `docs/03-design/data-model/courses.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-040, RN-041, RN-043, RN-046 | `crates/kampus-courses/tests/` | planned |
