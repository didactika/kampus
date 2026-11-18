---
id: UC-046
name: manageLanguagePacks
title: Install language packs and customise messages
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-046 Install language packs and customise messages — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageLanguagePacks/README.md) | **Analysis** | [Design](../../../03-design/use-cases/manageLanguagePacks/README.md) | [Implementation](../../../04-implementation/use-cases/manageLanguagePacks/README.md) | [Tests](../../../04-implementation/use-cases/manageLanguagePacks/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/manageLanguagePacks/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Languages admin, `POST /language-packs` | boundary | Collect the request |
| Pack installer | control | Apply RN-187 to RN-194 |
| Language pack, Translation, Customised message | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
