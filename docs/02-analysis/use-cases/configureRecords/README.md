---
id: UC-027
name: configureRecords
title: Configure the academic records of a course
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-027 Configure the academic records of a course — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/configureRecords/README.md) | **Analysis** | [Design](../../../03-design/use-cases/configureRecords/README.md) | [Implementation](../../../04-implementation/use-cases/configureRecords/README.md) | [Tests](../../../04-implementation/use-cases/configureRecords/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/configureRecords/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Records setup, `/courses/{id}/records` | boundary | Collect the request |
| Records structure service | control | Apply RN-100, RN-101, RN-106 to RN-108 |
| Grade category, Grade record, Scale, Formula | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
