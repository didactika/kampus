---
id: UC-034
name: downloadFile
title: Open or download a file
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-034 Open or download a file — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/downloadFile/README.md) | **Analysis** | [Design](../../../03-design/use-cases/downloadFile/README.md) | [Implementation](../../../04-implementation/use-cases/downloadFile/README.md) | [Tests](../../../04-implementation/use-cases/downloadFile/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/downloadFile/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| File link, `POST /files/{id}/url` | boundary | Collect the request |
| File access service | control | Apply RN-128 to RN-135 |
| File, Blob | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
