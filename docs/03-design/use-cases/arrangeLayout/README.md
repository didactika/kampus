---
id: UC-054
name: arrangeLayout
title: Arrange a page and its menus
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-054 Arrange a page and its menus — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/arrangeLayout/README.md) | [Analysis](../../../02-analysis/use-cases/arrangeLayout/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/arrangeLayout/README.md) | [Tests](../../../04-implementation/use-cases/arrangeLayout/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/arrangeLayout/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Layout and menu service | service | `crates/customise/src/layout.rs` |
| `/api/v1/access/scopes/{scope_id}/layouts`, `/menus` | API endpoints | `docs/03-design/api/customise.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| customise.layout, customise.placement, customise.menu, customise.menu_item | replace |
