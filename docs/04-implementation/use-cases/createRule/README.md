---
id: UC-056
name: createRule
title: Create a rule
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-056 Create a rule — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/createRule/README.md) | [Analysis](../../../02-analysis/use-cases/createRule/README.md) | [Design](../../../03-design/use-cases/createRule/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/createRule/README.md#tests) |

Design: [README](../../../03-design/use-cases/createRule/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Rule engine | service | `crates/kampus-customise/src/rules.rs` | no (to build) |
| `/api/v1/scopes/{scope_id}/rules` | API endpoints | `docs/03-design/api/customise.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-218 to RN-222 | `crates/kampus-customise/tests/` | planned |
