---
id: UC-052
name: renameTerms
title: Rename terms
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-052 Rename terms — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/renameTerms/README.md) | [Analysis](../../../02-analysis/use-cases/renameTerms/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/renameTerms/README.md) | [Tests](../../../04-implementation/use-cases/renameTerms/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/renameTerms/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Term resolver | service | `crates/kampus-customise/src/terms.rs` |
| `/api/v1/scopes/{scope_id}/terms` | API endpoints | `docs/03-design/api/customise.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| customise.term_override | insert, update, delete |
| customise.change | insert |
