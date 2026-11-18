---
id: UC-027
name: configureRecords
title: Configure the academic records of a course
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-027 Configure the academic records of a course — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/configureRecords/README.md) | [Analysis](../../../02-analysis/use-cases/configureRecords/README.md) | [Design](../../../03-design/use-cases/configureRecords/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/configureRecords/README.md#tests) |

Design: [README](../../../03-design/use-cases/configureRecords/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Records structure service | service | `crates/kampus-records/src/structure.rs` | no (to build) |
| Aggregation methods | service | `crates/kampus-records/src/aggregation.rs` | no (to build) |
| `/api/v1/courses/{course_id}/records/*`, `/api/v1/records/{record_id}` | API endpoints | `docs/03-design/api/records.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-100, RN-101, RN-106 to RN-108 | `crates/kampus-records/tests/` | planned |
