---
id: UC-043
name: exportMyData
title: Export my personal data
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-043 Export my personal data — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/exportMyData/README.md) | **Analysis** | [Design](../../../03-design/use-cases/exportMyData/README.md) | [Implementation](../../../04-implementation/use-cases/exportMyData/README.md) | [Tests](../../../04-implementation/use-cases/exportMyData/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/exportMyData/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Privacy page, `POST /privacy/requests` | boundary | Collect the request |
| Request coordinator | control | Apply RN-178 to RN-186 |
| Data request, Request item, Export package | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
