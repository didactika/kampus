---
id: UC-008
name: setUpSecondFactor
title: Set up a second factor
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-008 Set up a second factor — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/setUpSecondFactor/README.md) | [Analysis](../../../02-analysis/use-cases/setUpSecondFactor/README.md) | [Design](../../../03-design/use-cases/setUpSecondFactor/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/setUpSecondFactor/README.md#tests) |

Design: [README](../../../03-design/use-cases/setUpSecondFactor/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Second-factor service | service | `crates/identity/src/service/second_factor.rs` | no (to build) |
| `/api/v1/identity/me/second-factors` | API endpoints | `docs/03-design/api/identity.openapi.yaml` | no (to build) |
| `identity.second_factor`, `identity.passkey`, `identity.recovery_code` | tables | `docs/03-design/data-model/identity.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-029, RN-032 | `crates/identity/tests/` | planned |
