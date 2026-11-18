---
id: UC-041
name: changeSetting
title: Change a setting
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-041 Change a setting — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/changeSetting/README.md) | **Analysis** | [Design](../../../03-design/use-cases/changeSetting/README.md) | [Implementation](../../../04-implementation/use-cases/changeSetting/README.md) | [Tests](../../../04-implementation/use-cases/changeSetting/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/changeSetting/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Settings page, `PUT /scopes/{id}/settings/{key}` | boundary | Collect the request |
| Settings resolver | control | Apply RN-162 to RN-169 |
| Setting definition, Setting value, Setting change | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
