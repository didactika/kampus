---
id: UC-021
name: addActivity
title: Add an activity or material
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-021 Add an activity or material — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/addActivity/README.md) | **Analysis** | [Design](../../../03-design/use-cases/addActivity/README.md) | [Implementation](../../../04-implementation/use-cases/addActivity/README.md) | [Tests](../../../04-implementation/use-cases/addActivity/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/addActivity/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Course editor, `POST /sections/{id}/course-modules` | boundary | Collect the request |
| Course module service | control | Apply RN-075 to RN-078 |
| Course module, Activity type | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
