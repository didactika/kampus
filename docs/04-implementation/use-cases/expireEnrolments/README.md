---
id: UC-020
name: expireEnrolments
title: Expire enrolments
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-020 Expire enrolments — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/expireEnrolments/README.md) | [Analysis](../../../02-analysis/use-cases/expireEnrolments/README.md) | [Design](../../../03-design/use-cases/expireEnrolments/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/expireEnrolments/README.md#tests) |

Design: [README](../../../03-design/use-cases/expireEnrolments/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Expiry job | job | `crates/kampus-enrol/src/jobs/expire.rs` | no (to build) |
| `enrol.expiry_outcome` | table | `docs/03-design/data-model/enrolment.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-068, RN-069 | `crates/kampus-enrol/tests/` | planned |
