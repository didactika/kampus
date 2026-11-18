---
id: UC-050
name: managePlugin
title: Upgrade, disable or remove a plugin
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-050 Upgrade, disable or remove a plugin — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/managePlugin/README.md) | **Analysis** | [Design](../../../03-design/use-cases/managePlugin/README.md) | [Implementation](../../../04-implementation/use-cases/managePlugin/README.md) | [Tests](../../../04-implementation/use-cases/managePlugin/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/managePlugin/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Plugins admin, `/plugins/{id}` | boundary | Collect the request |
| Plugin lifecycle service | control | Apply RN-205 to RN-211 |
| Installation, Plugin version, Plugin failure | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
