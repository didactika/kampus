---
id: UC-014
name: manageGroups
title: Manage groups and group sets
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-014 Manage groups and group sets — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageGroups/README.md) | [Analysis](../../../02-analysis/use-cases/manageGroups/README.md) | [Design](../../../03-design/use-cases/manageGroups/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/manageGroups/README.md#tests) |

Design: [README](../../../03-design/use-cases/manageGroups/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Group service | service | `crates/courses/src/group.rs` | no (to build) |
| `/api/v1/courses/courses/{course_id}/groups`, `/api/v1/courses/groups/{group_id}/members`, group sets | API endpoints | `docs/03-design/api/courses.openapi.yaml` | no (to build) |
| `courses.group`, `courses.group_member`, `courses.group_set` | tables | `docs/03-design/data-model/courses.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-041, RN-053 to RN-058 | `crates/courses/tests/` | planned |
