---
id: UC-056
name: createRule
title: Create a rule
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-056 Create a rule — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/createRule/README.md) | **Analysis** | [Design](../../../03-design/use-cases/createRule/README.md) | [Implementation](../../../04-implementation/use-cases/createRule/README.md) | [Tests](../../../04-implementation/use-cases/createRule/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/createRule/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Rules page, `POST /scopes/{id}/rules` | boundary | Collect the request |
| Rule engine | control | Apply RN-218 to RN-222 |
| Rule, Rule run | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
