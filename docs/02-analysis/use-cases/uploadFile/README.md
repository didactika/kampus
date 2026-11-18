---
id: UC-033
name: uploadFile
title: Upload a file
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-033 Upload a file — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/uploadFile/README.md) | **Analysis** | [Design](../../../03-design/use-cases/uploadFile/README.md) | [Implementation](../../../04-implementation/use-cases/uploadFile/README.md) | [Tests](../../../04-implementation/use-cases/uploadFile/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/uploadFile/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| File picker, `POST /uploads` | boundary | Collect the request |
| Upload pipeline | control | Apply RN-126 to RN-134 |
| Upload session, Blob, File, Quota | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
