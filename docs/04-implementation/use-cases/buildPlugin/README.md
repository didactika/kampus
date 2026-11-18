---
id: UC-051
name: buildPlugin
title: Build and package a plugin
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-051 Build and package a plugin — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/buildPlugin/README.md) | [Analysis](../../../02-analysis/use-cases/buildPlugin/README.md) | [Design](../../../03-design/use-cases/buildPlugin/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/buildPlugin/README.md#tests) |

Design: [README](../../../03-design/use-cases/buildPlugin/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| SDK and test harness | tool | `crates/sdk`, `crates/plugin-host/src/harness.rs` | no (to build) |
| WIT packages | contract | `docs/03-design/sdk/kampus.wit` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-203 to RN-211 | `crates/plugin-host/tests/` | planned |
