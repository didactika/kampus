---
id: UC-023
name: grantException
title: Grant an exception
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-023 Grant an exception — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/grantException/README.md) | **Analysis** | [Design](../../../03-design/use-cases/grantException/README.md) | [Implementation](../../../04-implementation/use-cases/grantException/README.md) | [Tests](../../../04-implementation/use-cases/grantException/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/grantException/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Exceptions screen, `/course-modules/{id}/exceptions` | boundary | Collect the request |
| Exception service and resolver | control | Apply RN-084 to RN-087 |
| Exception | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
