---
id: UC-052
name: renameTerms
title: Rename terms
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-052 Rename terms — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/renameTerms/README.md) | **Analysis** | [Design](../../../03-design/use-cases/renameTerms/README.md) | [Implementation](../../../04-implementation/use-cases/renameTerms/README.md) | [Tests](../../../04-implementation/use-cases/renameTerms/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/renameTerms/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Terminology page, `PUT /scopes/{id}/terms/{code}` | boundary | Collect the request |
| Term resolver | control | Apply RN-212 to RN-222 |
| Term, Term override | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
