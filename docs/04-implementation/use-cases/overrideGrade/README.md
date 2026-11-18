---
id: UC-028
name: overrideGrade
title: Override or lock a grade
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-028 Override or lock a grade — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/overrideGrade/README.md) | [Analysis](../../../02-analysis/use-cases/overrideGrade/README.md) | [Design](../../../03-design/use-cases/overrideGrade/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/overrideGrade/README.md#tests) |

Design: [README](../../../03-design/use-cases/overrideGrade/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Grade service | service | `crates/records/src/grade.rs` | no (to build) |
| override and lock endpoints | API endpoints | `docs/03-design/api/records.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-097, RN-098 | `crates/records/tests/` | planned |
