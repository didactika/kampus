---
id: UC-032
name: setProgressException
title: Set a progress exception
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-032 Set a progress exception — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/setProgressException/README.md) | **Analysis** | [Design](../../../03-design/use-cases/setProgressException/README.md) | [Implementation](../../../04-implementation/use-cases/setProgressException/README.md) | [Tests](../../../04-implementation/use-cases/setProgressException/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/setProgressException/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Progress report, `/course-modules/{id}/progress/{account}/exception` | boundary | Collect the request |
| Progress exception service | control | Apply RN-118, RN-119 |
| Progress exception | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
