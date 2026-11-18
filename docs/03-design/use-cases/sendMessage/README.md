---
id: UC-038
name: sendMessage
title: Send a message
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-038 Send a message — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/sendMessage/README.md) | [Analysis](../../../02-analysis/use-cases/sendMessage/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/sendMessage/README.md) | [Tests](../../../04-implementation/use-cases/sendMessage/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/sendMessage/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Messaging service | service | `crates/kampus-notify/src/messaging.rs` |
| `/api/v1/conversations` | API endpoints | `docs/03-design/api/notify.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| notify.conversation, notify.member, notify.message | insert, update |
| notify.notification, notify.delivery | insert |
