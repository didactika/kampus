---
id: UC-054
name: arrangeLayout
title: Arrange a page and its menus
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-054 Arrange a page and its menus — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/arrangeLayout/README.md) | **Analysis** | [Design](../../../03-design/use-cases/arrangeLayout/README.md) | [Implementation](../../../04-implementation/use-cases/arrangeLayout/README.md) | [Tests](../../../04-implementation/use-cases/arrangeLayout/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/arrangeLayout/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Page editor, `PUT /scopes/{id}/layouts/{page_kind}` | boundary | Collect the request |
| Layout service | control | Apply RN-213 to RN-222 |
| Layout, Widget placement, Navigation menu | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
