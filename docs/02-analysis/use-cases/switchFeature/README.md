---
id: UC-057
name: switchFeature
title: Switch a feature or module on or off
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-057 Switch a feature or module on or off — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/switchFeature/README.md) | **Analysis** | [Design](../../../03-design/use-cases/switchFeature/README.md) | [Implementation](../../../04-implementation/use-cases/switchFeature/README.md) | [Tests](../../../04-implementation/use-cases/switchFeature/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/switchFeature/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Features page, `PUT /scopes/{id}/flags/{code}` | boundary | Collect the request |
| Flag resolver | control | Apply RN-213 to RN-220 |
| Feature flag | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
