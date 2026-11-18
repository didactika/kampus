---
id: UC-013
name: organiseSections
title: Organise the sections of a course
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-013 Organise the sections of a course — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/organiseSections/README.md) | [Analysis](../../../02-analysis/use-cases/organiseSections/README.md) | [Design](../../../03-design/use-cases/organiseSections/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/organiseSections/README.md#tests) |

Design: [README](../../../03-design/use-cases/organiseSections/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Section service | service | `crates/courses/src/section.rs` | no (to build) |
| `/api/v1/courses/{course_id}/sections`, `/api/v1/sections/{section_id}`, move | API endpoints | `docs/03-design/api/courses.openapi.yaml` | no (to build) |
| `courses.section` | table | `docs/03-design/data-model/courses.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-043, RN-045, RN-047, RN-048 | `crates/courses/tests/` | planned |
