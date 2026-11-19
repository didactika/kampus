---
id: UC-047
name: registerApiClient
title: Register an API client
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-047 Register an API client — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/registerApiClient/README.md) | [Analysis](../../../02-analysis/use-cases/registerApiClient/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/registerApiClient/README.md) | [Tests](../../../04-implementation/use-cases/registerApiClient/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/registerApiClient/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Client registry and token service | service | `crates/api/src/clients.rs` |
| `/api/v1/integrations/api-clients`, `/oauth/*` | API endpoints | `docs/03-design/api/integrations.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| api.client, api.authorisation, api.refresh_token | insert, update |
