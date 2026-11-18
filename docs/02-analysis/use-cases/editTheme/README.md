---
id: UC-055
name: editTheme
title: Edit the theme
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-055 Edit the theme — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/editTheme/README.md) | **Analysis** | [Design](../../../03-design/use-cases/editTheme/README.md) | [Implementation](../../../04-implementation/use-cases/editTheme/README.md) | [Tests](../../../04-implementation/use-cases/editTheme/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/editTheme/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Theme editor, `PUT /scopes/{id}/theme` | boundary | Collect the request |
| Theme service | control | Apply RN-213 to RN-217 |
| Theme | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
