---
id: UC-049
name: installPlugin
title: Install a plugin
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-049 Install a plugin — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/installPlugin/README.md) | **Analysis** | [Design](../../../03-design/use-cases/installPlugin/README.md) | [Implementation](../../../04-implementation/use-cases/installPlugin/README.md) | [Tests](../../../04-implementation/use-cases/installPlugin/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/installPlugin/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Plugins admin, `POST /plugins` | boundary | Collect the request |
| Plugin installer | control | Apply RN-203 to RN-207 |
| Plugin, Plugin version, Capability grant, Installation | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
