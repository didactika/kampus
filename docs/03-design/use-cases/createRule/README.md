---
id: UC-056
name: createRule
title: Create a rule
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-056 Create a rule — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/createRule/README.md) | [Analysis](../../../02-analysis/use-cases/createRule/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/createRule/README.md) | [Tests](../../../04-implementation/use-cases/createRule/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/createRule/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Rule engine | service | `crates/customise/src/rules.rs` |
| `/api/v1/customise/scopes/{scope_id}/rules` | API endpoints | `docs/03-design/api/customise.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| customise.rule | insert, update |
| customise.rule_run, tasks.job | insert |
