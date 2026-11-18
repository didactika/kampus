---
id: UC-044
name: processDeletionRequest
title: Process a deletion request
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-044 Process a deletion request — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/processDeletionRequest/README.md) | [Analysis](../../../02-analysis/use-cases/processDeletionRequest/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/processDeletionRequest/README.md) | [Tests](../../../04-implementation/use-cases/processDeletionRequest/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/processDeletionRequest/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Request coordinator and retention job | service | `crates/privacy/src/requests.rs` |
| `/api/v1/privacy/requests/{request_id}/approve` | API endpoint | `docs/03-design/api/privacy.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| privacy.request, privacy.request_item | update |
| identity.account and every module's personal data | update, delete (through their handlers) |
