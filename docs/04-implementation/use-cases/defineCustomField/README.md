---
id: UC-053
name: defineCustomField
title: Define a custom field
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-053 Define a custom field — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/defineCustomField/README.md) | [Analysis](../../../02-analysis/use-cases/defineCustomField/README.md) | [Design](../../../03-design/use-cases/defineCustomField/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/defineCustomField/README.md#tests) |

Design: [README](../../../03-design/use-cases/defineCustomField/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Custom field service | service | `crates/kampus-customise/src/fields.rs` | no (to build) |
| `/api/v1/scopes/{scope_id}/fields` | API endpoints | `docs/03-design/api/customise.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-213 to RN-215 | `crates/kampus-customise/tests/` | planned |
