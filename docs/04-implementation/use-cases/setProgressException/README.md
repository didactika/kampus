---
id: UC-032
name: setProgressException
title: Set a progress exception
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-032 Set a progress exception — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/setProgressException/README.md) | [Analysis](../../../02-analysis/use-cases/setProgressException/README.md) | [Design](../../../03-design/use-cases/setProgressException/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/setProgressException/README.md#tests) |

Design: [README](../../../03-design/use-cases/setProgressException/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Progress exception service | service | `crates/kampus-progress/src/exception.rs` | no (to build) |
| exception endpoint | API endpoint | `docs/03-design/api/progress.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-118, RN-119 | `crates/kampus-progress/tests/` | planned |
