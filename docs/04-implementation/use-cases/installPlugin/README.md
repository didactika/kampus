---
id: UC-049
name: installPlugin
title: Install a plugin
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-049 Install a plugin — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/installPlugin/README.md) | [Analysis](../../../02-analysis/use-cases/installPlugin/README.md) | [Design](../../../03-design/use-cases/installPlugin/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/installPlugin/README.md#tests) |

Design: [README](../../../03-design/use-cases/installPlugin/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Plugin installer | service | `crates/plugin-host/src/install.rs` | no (to build) |
| `/api/v1/plugins/plugins` | API endpoints | `docs/03-design/api/plugins.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-203 to RN-207 | `crates/plugin-host/tests/` | planned |
