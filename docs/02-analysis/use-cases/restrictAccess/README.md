---
id: UC-022
name: restrictAccess
title: Restrict access to an activity or section
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-022 Restrict access to an activity or section — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/restrictAccess/README.md) | **Analysis** | [Design](../../../03-design/use-cases/restrictAccess/README.md) | [Implementation](../../../04-implementation/use-cases/restrictAccess/README.md) | [Tests](../../../04-implementation/use-cases/restrictAccess/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/restrictAccess/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Conditions editor, `PUT /course-modules/{id}/conditions` | boundary | Collect the request |
| Condition service | control | Apply RN-080 to RN-083 |
| Access condition | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
