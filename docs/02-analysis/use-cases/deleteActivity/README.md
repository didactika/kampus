---
id: UC-025
name: deleteActivity
title: Delete and restore an activity
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-025 Delete and restore an activity — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/deleteActivity/README.md) | **Analysis** | [Design](../../../03-design/use-cases/deleteActivity/README.md) | [Implementation](../../../04-implementation/use-cases/deleteActivity/README.md) | [Tests](../../../04-implementation/use-cases/deleteActivity/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/deleteActivity/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Course editor and bin, `/course-modules/{id}`, `/bin-items/{id}/*` | boundary | Collect the request |
| Bin service and purge job | control | Apply RN-088, RN-089 |
| Course module, Recycle bin item | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
