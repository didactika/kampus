---
id: UC-041
name: changeSetting
title: Change a setting
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-041 Change a setting — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/changeSetting/README.md) | [Analysis](../../../02-analysis/use-cases/changeSetting/README.md) | [Design](../../../03-design/use-cases/changeSetting/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/changeSetting/README.md#tests) |

Design: [README](../../../03-design/use-cases/changeSetting/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Settings resolver | service | `crates/kampus-settings/src/resolve.rs` | no (to build) |
| `/api/v1/scopes/{scope_id}/settings` | API endpoints | `docs/03-design/api/settings.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-162 to RN-169 | `crates/kampus-settings/tests/` | planned |
