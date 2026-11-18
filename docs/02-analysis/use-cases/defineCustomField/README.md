---
id: UC-053
name: defineCustomField
title: Define a custom field
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-053 Define a custom field — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/defineCustomField/README.md) | **Analysis** | [Design](../../../03-design/use-cases/defineCustomField/README.md) | [Implementation](../../../04-implementation/use-cases/defineCustomField/README.md) | [Tests](../../../04-implementation/use-cases/defineCustomField/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/defineCustomField/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Custom fields page, `POST /scopes/{id}/fields` | boundary | Collect the request |
| Custom field service | control | Apply RN-213 to RN-215 |
| Custom field, Custom field value | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
