---
id: UC-029
name: viewGrades
title: See grades
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-029 See grades — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/viewGrades/README.md) | **Analysis** | [Design](../../../03-design/use-cases/viewGrades/README.md) | [Implementation](../../../04-implementation/use-cases/viewGrades/README.md) | [Tests](../../../04-implementation/use-cases/viewGrades/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/viewGrades/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| My grades, grader report, export | boundary | Collect the request |
| Grade read service | control | Apply RN-103 to RN-105, RN-109 |
| Grade, Grade record | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
