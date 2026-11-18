---
id: UC-040
name: subscribeCalendar
title: Subscribe to or export a calendar
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-040 Subscribe to or export a calendar — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/subscribeCalendar/README.md) | **Analysis** | [Design](../../../03-design/use-cases/subscribeCalendar/README.md) | [Implementation](../../../04-implementation/use-cases/subscribeCalendar/README.md) | [Tests](../../../04-implementation/use-cases/subscribeCalendar/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/subscribeCalendar/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Calendar settings, `POST /scopes/{id}/calendar-subscriptions` | boundary | Collect the request |
| Subscription refresher | control | Apply RN-159 to RN-161 |
| Subscription, Imported entry, Feed token | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
