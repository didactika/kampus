---
id: UC-024
name: copyActivity
title: Copy or move an activity
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-024 Copy or move an activity — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/copyActivity/README.md) | **Analysis** | [Design](../../../03-design/use-cases/copyActivity/README.md) | [Implementation](../../../04-implementation/use-cases/copyActivity/README.md) | [Tests](../../../04-implementation/use-cases/copyActivity/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/copyActivity/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Course editor, `/course-modules/{id}/copy`, `/move` | boundary | Collect the request |
| Course module service | control | Apply RN-076, RN-077, RN-090 |
| Course module | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
