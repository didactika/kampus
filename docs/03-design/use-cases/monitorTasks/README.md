---
id: UC-042
name: monitorTasks
title: Monitor background tasks
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-042 Monitor background tasks — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/monitorTasks/README.md) | [Analysis](../../../02-analysis/use-cases/monitorTasks/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/monitorTasks/README.md) | [Tests](../../../04-implementation/use-cases/monitorTasks/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/monitorTasks/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Job runner and scheduler | service | `crates/tasks/src/runner.rs` |
| `/api/v1/jobs`, `/api/v1/schedules` | API endpoints | `docs/03-design/api/tasks.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| tasks.job, tasks.attempt | select, update |
| tasks.schedule, tasks.schedule_tick | update, insert |
