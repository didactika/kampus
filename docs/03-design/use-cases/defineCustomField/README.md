---
id: UC-053
name: defineCustomField
title: Define a custom field
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-053 Define a custom field — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/defineCustomField/README.md) | [Analysis](../../../02-analysis/use-cases/defineCustomField/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/defineCustomField/README.md) | [Tests](../../../04-implementation/use-cases/defineCustomField/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/defineCustomField/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Custom field service | service | `crates/kampus-customise/src/fields.rs` |
| `/api/v1/scopes/{scope_id}/fields` | API endpoints | `docs/03-design/api/customise.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| customise.field, customise.field_choice | insert, update |
| customise.field_value and the owner's link table | insert, update |
