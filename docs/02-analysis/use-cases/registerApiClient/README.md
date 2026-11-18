---
id: UC-047
name: registerApiClient
title: Register an API client
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-047 Register an API client — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/registerApiClient/README.md) | **Analysis** | [Design](../../../03-design/use-cases/registerApiClient/README.md) | [Implementation](../../../04-implementation/use-cases/registerApiClient/README.md) | [Tests](../../../04-implementation/use-cases/registerApiClient/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/registerApiClient/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| API clients screen, `POST /api-clients` | boundary | Collect the request |
| Client registry and token service | control | Apply RN-195 to RN-198 |
| API client, API scope, Authorisation | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
