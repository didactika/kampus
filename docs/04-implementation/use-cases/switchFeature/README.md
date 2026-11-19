---
id: UC-057
name: switchFeature
title: Switch a feature or module on or off
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-057 Switch a feature or module on or off — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/switchFeature/README.md) | [Analysis](../../../02-analysis/use-cases/switchFeature/README.md) | [Design](../../../03-design/use-cases/switchFeature/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/switchFeature/README.md#tests) |

Design: [README](../../../03-design/use-cases/switchFeature/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Flag resolver | service | `crates/customise/src/flags.rs` | no (to build) |
| `/api/v1/customise/scopes/{scope_id}/flags` | API endpoints | `docs/03-design/api/customise.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-213 to RN-220 | `crates/customise/tests/` | planned |
