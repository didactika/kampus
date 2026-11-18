---
id: UC-043
name: exportMyData
title: Export my personal data
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-043 Export my personal data — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/exportMyData/README.md) | [Analysis](../../../02-analysis/use-cases/exportMyData/README.md) | [Design](../../../03-design/use-cases/exportMyData/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/exportMyData/README.md#tests) |

Design: [README](../../../03-design/use-cases/exportMyData/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Request coordinator | service | `crates/kampus-privacy/src/requests.rs` | no (to build) |
| `/api/v1/privacy/requests` | API endpoints | `docs/03-design/api/privacy.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-178 to RN-186 | `crates/kampus-privacy/tests/` | planned |
