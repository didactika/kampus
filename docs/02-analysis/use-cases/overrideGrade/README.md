---
id: UC-028
name: overrideGrade
title: Override or lock a grade
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-028 Override or lock a grade — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/overrideGrade/README.md) | **Analysis** | [Design](../../../03-design/use-cases/overrideGrade/README.md) | [Implementation](../../../04-implementation/use-cases/overrideGrade/README.md) | [Tests](../../../04-implementation/use-cases/overrideGrade/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/overrideGrade/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Grader report, `/records/{id}/grades/{account}/override`, `/lock` | boundary | Collect the request |
| Grade service | control | Apply RN-097, RN-098 |
| Grade, Grade entry | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
