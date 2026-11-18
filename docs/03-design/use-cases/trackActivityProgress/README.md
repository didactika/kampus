---
id: UC-030
name: trackActivityProgress
title: Track activity progress
status: proposed
origin: requirements
sourceOfTruth: model
---

# UC-030 Track activity progress — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/trackActivityProgress/README.md) | [Analysis](../../../02-analysis/use-cases/trackActivityProgress/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/trackActivityProgress/README.md) | [Tests](../../../04-implementation/use-cases/trackActivityProgress/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/trackActivityProgress/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Progress evaluator | service | `crates/kampus-progress/src/activity.rs` |
| progress endpoints | API endpoints | `docs/03-design/api/progress.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| progress.activity_state, progress.activity_history | upsert, insert |
