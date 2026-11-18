---
id: UC-039
name: viewCalendar
title: See the calendar
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-039 See the calendar — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/viewCalendar/README.md) | **Analysis** | [Design](../../../03-design/use-cases/viewCalendar/README.md) | [Implementation](../../../04-implementation/use-cases/viewCalendar/README.md) | [Tests](../../../04-implementation/use-cases/viewCalendar/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/viewCalendar/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Calendar page, `GET /calendar` | boundary | Collect the request |
| Calendar view service | control | Apply RN-156 to RN-160 |
| Calendar entry, Activity date, Recurrence | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
