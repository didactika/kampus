---
id: UC-048
name: subscribeWebhook
title: Subscribe a webhook
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-048 Subscribe a webhook — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/subscribeWebhook/README.md) | [Analysis](../../../02-analysis/use-cases/subscribeWebhook/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/subscribeWebhook/README.md) | [Tests](../../../04-implementation/use-cases/subscribeWebhook/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/subscribeWebhook/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Webhook dispatcher | service | `crates/kampus-api/src/webhooks.rs` |
| `/api/v1/api-clients/{client_id}/webhooks` | API endpoints | `docs/03-design/api/integrations.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| api.webhook | insert, update |
| api.webhook_delivery | insert, update |
