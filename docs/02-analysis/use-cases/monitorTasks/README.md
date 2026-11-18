---
id: UC-042
name: monitorTasks
title: Monitor background tasks
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-042 Monitor background tasks — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/monitorTasks/README.md) | **Analysis** | [Design](../../../03-design/use-cases/monitorTasks/README.md) | [Implementation](../../../04-implementation/use-cases/monitorTasks/README.md) | [Tests](../../../04-implementation/use-cases/monitorTasks/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/monitorTasks/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Task monitor, `GET /jobs` | boundary | Collect the request |
| Job runner and scheduler | control | Apply RN-170 to RN-177 |
| Job, Job attempt, Schedule | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
