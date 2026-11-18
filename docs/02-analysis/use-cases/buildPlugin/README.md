---
id: UC-051
name: buildPlugin
title: Build and package a plugin
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-051 Build and package a plugin — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/buildPlugin/README.md) | **Analysis** | [Design](../../../03-design/use-cases/buildPlugin/README.md) | [Implementation](../../../04-implementation/use-cases/buildPlugin/README.md) | [Tests](../../../04-implementation/use-cases/buildPlugin/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/buildPlugin/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| `kampus plugin` commands | boundary | Collect the request |
| Host test harness | control | Apply RN-203 to RN-211 |
| Plugin type, Capability, Plugin version | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
