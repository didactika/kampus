---
id: UC-032
name: setProgressException
title: Set a progress exception
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-032 Set a progress exception — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/setProgressException/README.md) | [Analysis](../../../02-analysis/use-cases/setProgressException/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/setProgressException/README.md) | [Tests](../../../04-implementation/use-cases/setProgressException/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/setProgressException/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Progress exception service | service | `crates/progress/src/exception.rs` |
| exception endpoint | API endpoint | `docs/03-design/api/progress.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| progress.activity_exception, progress.activity_history | upsert, insert |
