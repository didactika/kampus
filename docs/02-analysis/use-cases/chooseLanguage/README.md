---
id: UC-045
name: chooseLanguage
title: Choose a language
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-045 Choose a language — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/chooseLanguage/README.md) | **Analysis** | [Design](../../../03-design/use-cases/chooseLanguage/README.md) | [Implementation](../../../04-implementation/use-cases/chooseLanguage/README.md) | [Tests](../../../04-implementation/use-cases/chooseLanguage/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/chooseLanguage/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Language menu, `PUT /me/language` | boundary | Collect the request |
| Language resolver | control | Apply RN-189 to RN-192 |
| Language, Profile | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
