---
id: UC-054
name: arrangeLayout
title: Arrange a page and its menus
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-054 Arrange a page and its menus — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/arrangeLayout/README.md) | [Analysis](../../../02-analysis/use-cases/arrangeLayout/README.md) | [Design](../../../03-design/use-cases/arrangeLayout/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/arrangeLayout/README.md#tests) |

Design: [README](../../../03-design/use-cases/arrangeLayout/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Layout and menu service | service | `crates/kampus-customise/src/layout.rs` | no (to build) |
| `/api/v1/scopes/{scope_id}/layouts`, `/menus` | API endpoints | `docs/03-design/api/customise.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-213 to RN-222 | `crates/kampus-customise/tests/` | planned |
