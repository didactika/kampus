---
id: UC-029
name: viewGrades
title: See grades
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-029 See grades — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/viewGrades/README.md) | [Analysis](../../../02-analysis/use-cases/viewGrades/README.md) | [Design](../../../03-design/use-cases/viewGrades/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/viewGrades/README.md#tests) |

Design: [README](../../../03-design/use-cases/viewGrades/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Grade read service | service | `crates/records/src/read.rs` | no (to build) |
| `/my-grades`, `/grader-report`, export | API endpoints | `docs/03-design/api/records.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-103 to RN-105, RN-109 | `crates/records/tests/` | planned |
