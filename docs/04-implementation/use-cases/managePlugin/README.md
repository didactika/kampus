---
id: UC-050
name: managePlugin
title: Upgrade, disable or remove a plugin
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-050 Upgrade, disable or remove a plugin — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/managePlugin/README.md) | [Analysis](../../../02-analysis/use-cases/managePlugin/README.md) | [Design](../../../03-design/use-cases/managePlugin/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/managePlugin/README.md#tests) |

Design: [README](../../../03-design/use-cases/managePlugin/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Plugin lifecycle service | service | `crates/kampus-plugin-host/src/lifecycle.rs` | no (to build) |
| `/api/v1/plugins/{plugin_id}` | API endpoints | `docs/03-design/api/plugins.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-205 to RN-211 | `crates/kampus-plugin-host/tests/` | planned |
