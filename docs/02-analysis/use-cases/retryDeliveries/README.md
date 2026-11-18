---
id: UC-035
name: retryDeliveries
title: Review and retry failed event deliveries
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-035 Review and retry failed event deliveries — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/retryDeliveries/README.md) | **Analysis** | [Design](../../../03-design/use-cases/retryDeliveries/README.md) | [Implementation](../../../04-implementation/use-cases/retryDeliveries/README.md) | [Tests](../../../04-implementation/use-cases/retryDeliveries/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/retryDeliveries/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Deliveries screen, `GET /deliveries/dead` | boundary | Collect the request |
| Dispatcher | control | Apply RN-137 to RN-142 |
| Delivery, Listener, Event | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
