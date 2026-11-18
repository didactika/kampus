---
id: UC-026
name: receiveGrade
title: Receive a grade from an activity
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-026 Receive a grade from an activity — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/receiveGrade/README.md) | **Analysis** | [Design](../../../03-design/use-cases/receiveGrade/README.md) | [Implementation](../../../04-implementation/use-cases/receiveGrade/README.md) | [Tests](../../../04-implementation/use-cases/receiveGrade/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/receiveGrade/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Records service API (called by activity types) | boundary | Collect the request |
| Grade service and recalculation job | control | Apply RN-096 to RN-099 |
| Grade record, Grade entry, Grade | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
