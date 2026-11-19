---
id: UC-041
name: changeSetting
title: Change a setting
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-041 Change a setting — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/changeSetting/README.md) | [Analysis](../../../02-analysis/use-cases/changeSetting/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/changeSetting/README.md) | [Tests](../../../04-implementation/use-cases/changeSetting/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/changeSetting/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Settings resolver | service | `crates/settings/src/resolve.rs` |
| `/api/v1/settings/scopes/{scope_id}/settings` | API endpoints | `docs/03-design/api/settings.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| settings.value | insert, update, delete |
| settings.change | insert |
