---
id: UC-044
name: processDeletionRequest
title: Process a deletion request
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-044 Process a deletion request — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/processDeletionRequest/README.md) | [Analysis](../../../02-analysis/use-cases/processDeletionRequest/README.md) | [Design](../../../03-design/use-cases/processDeletionRequest/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/processDeletionRequest/README.md#tests) |

Design: [README](../../../03-design/use-cases/processDeletionRequest/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Request coordinator and retention job | service | `crates/privacy/src/requests.rs` | no (to build) |
| `/api/v1/privacy/requests/{request_id}/approve` | API endpoint | `docs/03-design/api/privacy.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-180 to RN-185 | `crates/privacy/tests/` | planned |
