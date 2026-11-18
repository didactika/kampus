---
id: UC-048
name: subscribeWebhook
title: Subscribe a webhook
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-048 Subscribe a webhook — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/subscribeWebhook/README.md) | **Analysis** | [Design](../../../03-design/use-cases/subscribeWebhook/README.md) | [Implementation](../../../04-implementation/use-cases/subscribeWebhook/README.md) | [Tests](../../../04-implementation/use-cases/subscribeWebhook/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/subscribeWebhook/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Webhooks screen, `POST /api-clients/{id}/webhooks` | boundary | Collect the request |
| Webhook dispatcher | control | Apply RN-199 to RN-201 |
| Webhook, Webhook delivery | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
