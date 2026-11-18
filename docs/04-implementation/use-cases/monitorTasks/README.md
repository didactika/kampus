---
id: UC-042
name: monitorTasks
title: Monitor background tasks
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-042 Monitor background tasks — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/monitorTasks/README.md) | [Analysis](../../../02-analysis/use-cases/monitorTasks/README.md) | [Design](../../../03-design/use-cases/monitorTasks/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/monitorTasks/README.md#tests) |

Design: [README](../../../03-design/use-cases/monitorTasks/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Job runner and scheduler | service | `crates/kampus-tasks/src/runner.rs` | no (to build) |
| `/api/v1/jobs`, `/api/v1/schedules` | API endpoints | `docs/03-design/api/tasks.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-170 to RN-177 | `crates/kampus-tasks/tests/` | planned |
