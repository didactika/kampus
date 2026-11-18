---
id: UC-044
name: processDeletionRequest
title: Process a deletion request
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-044 Process a deletion request — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/processDeletionRequest/README.md) | **Analysis** | [Design](../../../03-design/use-cases/processDeletionRequest/README.md) | [Implementation](../../../04-implementation/use-cases/processDeletionRequest/README.md) | [Tests](../../../04-implementation/use-cases/processDeletionRequest/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/processDeletionRequest/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Data requests screen, `POST /privacy/requests/{id}/approve` | boundary | Collect the request |
| Request coordinator | control | Apply RN-180 to RN-185 |
| Data request, Request item, Legal hold | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
