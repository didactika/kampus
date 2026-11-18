---
id: UC-060
name: takeGuidedTour
title: Take a guided tour or follow the onboarding checklist
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-060 Take a guided tour or follow the onboarding checklist — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/takeGuidedTour/README.md) | [Analysis](../../../02-analysis/use-cases/takeGuidedTour/README.md) | [Design](../../../03-design/use-cases/takeGuidedTour/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/takeGuidedTour/README.md#tests) |

Design: [README](../../../03-design/use-cases/takeGuidedTour/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Tour runner and checklist | UI | `web/src/shell/tours/` | no (to build) |
| Onboarding progress | service | `crates/users/src/onboarding.rs` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | HB-05 to HB-07 | `web/tests/help/` | planned |
