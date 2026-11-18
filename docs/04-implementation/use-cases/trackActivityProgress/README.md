---
id: UC-030
name: trackActivityProgress
title: Track activity progress
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-030 Track activity progress — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/trackActivityProgress/README.md) | [Analysis](../../../02-analysis/use-cases/trackActivityProgress/README.md) | [Design](../../../03-design/use-cases/trackActivityProgress/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/trackActivityProgress/README.md#tests) |

Design: [README](../../../03-design/use-cases/trackActivityProgress/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Progress evaluator | service | `crates/kampus-progress/src/activity.rs` | no (to build) |
| progress endpoints | API endpoints | `docs/03-design/api/progress.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-115, RN-116, RN-123 | `crates/kampus-progress/tests/` | planned |
