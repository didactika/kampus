---
id: UC-030
name: trackActivityProgress
title: Track activity progress
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-030 Track activity progress — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/trackActivityProgress/README.md) | **Analysis** | [Design](../../../03-design/use-cases/trackActivityProgress/README.md) | [Implementation](../../../04-implementation/use-cases/trackActivityProgress/README.md) | [Tests](../../../04-implementation/use-cases/trackActivityProgress/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/trackActivityProgress/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Event listeners, `/course-modules/{id}/progress/me` | boundary | Collect the request |
| Progress evaluator | control | Apply RN-115, RN-116, RN-123 |
| Progress rule, Activity progress | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
