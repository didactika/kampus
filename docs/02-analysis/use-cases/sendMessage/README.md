---
id: UC-038
name: sendMessage
title: Send a message
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-038 Send a message — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/sendMessage/README.md) | **Analysis** | [Design](../../../03-design/use-cases/sendMessage/README.md) | [Implementation](../../../04-implementation/use-cases/sendMessage/README.md) | [Tests](../../../04-implementation/use-cases/sendMessage/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/sendMessage/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Messaging drawer, `POST /conversations/{id}/messages` | boundary | Collect the request |
| Messaging service | control | Apply RN-152 to RN-155 |
| Conversation, Member, Message | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
