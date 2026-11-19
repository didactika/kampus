---
id: UC-055
name: editTheme
title: Edit the theme
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-055 Edit the theme — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/editTheme/README.md) | [Analysis](../../../02-analysis/use-cases/editTheme/README.md) | [Design](../../../03-design/use-cases/editTheme/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/editTheme/README.md#tests) |

Design: [README](../../../03-design/use-cases/editTheme/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Theme service and contrast checker | service | `crates/customise/src/theme.rs` | no (to build) |
| `/api/v1/customise/scopes/{scope_id}/theme` | API endpoint | `docs/03-design/api/customise.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-213 to RN-217 | `crates/customise/tests/` | planned |
