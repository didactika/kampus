---
id: UC-048
name: subscribeWebhook
title: Subscribe a webhook
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-048 Subscribe a webhook — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/subscribeWebhook/README.md) | [Analysis](../../../02-analysis/use-cases/subscribeWebhook/README.md) | [Design](../../../03-design/use-cases/subscribeWebhook/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/subscribeWebhook/README.md#tests) |

Design: [README](../../../03-design/use-cases/subscribeWebhook/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Webhook dispatcher | service | `crates/kampus-api/src/webhooks.rs` | no (to build) |
| `/api/v1/api-clients/{client_id}/webhooks` | API endpoints | `docs/03-design/api/integrations.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-199 to RN-201 | `crates/kampus-api/tests/` | planned |
