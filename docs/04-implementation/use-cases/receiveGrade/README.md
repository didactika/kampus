---
id: UC-026
name: receiveGrade
title: Receive a grade from an activity
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-026 Receive a grade from an activity — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/receiveGrade/README.md) | [Analysis](../../../02-analysis/use-cases/receiveGrade/README.md) | [Design](../../../03-design/use-cases/receiveGrade/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/receiveGrade/README.md#tests) |

Design: [README](../../../03-design/use-cases/receiveGrade/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Grade service | service | `crates/records/src/grade.rs` | no (to build) |
| Recalculation job | job | `crates/records/src/jobs/recalculate.rs` | no (to build) |
| `PUT /api/v1/records/records/{record_id}/grades` | API endpoint | `docs/03-design/api/records.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-096 to RN-099 | `crates/records/tests/` | planned |
