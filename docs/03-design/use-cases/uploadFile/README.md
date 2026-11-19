---
id: UC-033
name: uploadFile
title: Upload a file
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-033 Upload a file — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/uploadFile/README.md) | [Analysis](../../../02-analysis/use-cases/uploadFile/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/uploadFile/README.md) | [Tests](../../../04-implementation/use-cases/uploadFile/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/uploadFile/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Upload pipeline | service | `crates/files/src/upload.rs` |
| `/api/v1/files/uploads` | API endpoints | `docs/03-design/api/files.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| files.upload_session | insert, update |
| files.blob, files.file, files.quota, the owner's link table | insert, update |
