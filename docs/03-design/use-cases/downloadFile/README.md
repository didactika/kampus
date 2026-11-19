---
id: UC-034
name: downloadFile
title: Open or download a file
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-034 Open or download a file — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/downloadFile/README.md) | [Analysis](../../../02-analysis/use-cases/downloadFile/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/downloadFile/README.md) | [Tests](../../../04-implementation/use-cases/downloadFile/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/downloadFile/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| File access service | service | `crates/files/src/access.rs` |
| `/api/v1/files/files/{file_id}/url` | API endpoint | `docs/03-design/api/files.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| files.file, files.blob | select |
| files.conversion | select, insert |
