---
id: UC-055
name: editTheme
title: Edit the theme
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-055 Edit the theme — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/editTheme/README.md) | [Analysis](../../../02-analysis/use-cases/editTheme/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/editTheme/README.md) | [Tests](../../../04-implementation/use-cases/editTheme/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/editTheme/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Theme service and contrast checker | service | `crates/customise/src/theme.rs` |
| `/api/v1/scopes/{scope_id}/theme` | API endpoint | `docs/03-design/api/customise.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| customise.theme | insert, update |
