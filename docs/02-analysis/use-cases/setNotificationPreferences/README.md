---
id: UC-037
name: setNotificationPreferences
title: Choose how to be notified
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-037 Choose how to be notified — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/setNotificationPreferences/README.md) | **Analysis** | [Design](../../../03-design/use-cases/setNotificationPreferences/README.md) | [Implementation](../../../04-implementation/use-cases/setNotificationPreferences/README.md) | [Tests](../../../04-implementation/use-cases/setNotificationPreferences/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/setNotificationPreferences/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Preferences page, `PUT /notification-preferences` | boundary | Collect the request |
| Preference resolver | control | Apply RN-147 to RN-151 |
| Notification type, Preference, Campus default | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
