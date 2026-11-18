---
id: UC-047
name: registerApiClient
title: Register an API client
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-047 Register an API client — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/registerApiClient/README.md) | [Analysis](../../../02-analysis/use-cases/registerApiClient/README.md) | [Design](../../../03-design/use-cases/registerApiClient/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/registerApiClient/README.md#tests) |

Design: [README](../../../03-design/use-cases/registerApiClient/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Client registry and token service | service | `crates/kampus-api/src/clients.rs` | no (to build) |
| `/api/v1/api-clients`, `/oauth/*` | API endpoints | `docs/03-design/api/integrations.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-195 to RN-198 | `crates/kampus-api/tests/` | planned |
