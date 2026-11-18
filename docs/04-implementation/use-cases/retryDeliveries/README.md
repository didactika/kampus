---
id: UC-035
name: retryDeliveries
title: Review and retry failed event deliveries
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-035 Review and retry failed event deliveries — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/retryDeliveries/README.md) | [Analysis](../../../02-analysis/use-cases/retryDeliveries/README.md) | [Design](../../../03-design/use-cases/retryDeliveries/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/retryDeliveries/README.md#tests) |

Design: [README](../../../03-design/use-cases/retryDeliveries/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Dispatcher and delivery store | service | `crates/events/src/dispatch.rs` | no (to build) |
| `/api/v1/deliveries` | API endpoints | `docs/03-design/api/events.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-137 to RN-142 | `crates/events/tests/` | planned |
