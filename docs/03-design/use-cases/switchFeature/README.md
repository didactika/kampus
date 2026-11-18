---
id: UC-057
name: switchFeature
title: Switch a feature or module on or off
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-057 Switch a feature or module on or off — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/switchFeature/README.md) | [Analysis](../../../02-analysis/use-cases/switchFeature/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/switchFeature/README.md) | [Tests](../../../04-implementation/use-cases/switchFeature/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/switchFeature/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Flag resolver | service | `crates/customise/src/flags.rs` |
| `/api/v1/scopes/{scope_id}/flags` | API endpoints | `docs/03-design/api/customise.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| customise.flag_value | insert, update |
