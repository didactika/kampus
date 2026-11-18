---
id: UC-035
name: retryDeliveries
title: Review and retry failed event deliveries
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-035 Review and retry failed event deliveries — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/retryDeliveries/README.md) | [Analysis](../../../02-analysis/use-cases/retryDeliveries/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/retryDeliveries/README.md) | [Tests](../../../04-implementation/use-cases/retryDeliveries/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/retryDeliveries/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Dispatcher and delivery store | service | `crates/kampus-events/src/dispatch.rs` |
| `/api/v1/deliveries` | API endpoints | `docs/03-design/api/events.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| events.delivery | select, update |
