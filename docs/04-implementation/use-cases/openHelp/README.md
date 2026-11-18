---
id: UC-059
name: openHelp
title: Open help for the current screen
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-059 Open help for the current screen — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/openHelp/README.md) | [Analysis](../../../02-analysis/use-cases/openHelp/README.md) | [Design](../../../03-design/use-cases/openHelp/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/openHelp/README.md#tests) |

Design: [README](../../../03-design/use-cases/openHelp/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Help panel and resolver | UI | `web/src/shell/help/` | no (to build) |
| Help topics of the handbook | content | `handbook/` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | HB-01 to HB-04 | `web/tests/help/` | planned |
