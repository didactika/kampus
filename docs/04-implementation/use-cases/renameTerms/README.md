---
id: UC-052
name: renameTerms
title: Rename terms
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-052 Rename terms — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/renameTerms/README.md) | [Analysis](../../../02-analysis/use-cases/renameTerms/README.md) | [Design](../../../03-design/use-cases/renameTerms/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/renameTerms/README.md#tests) |

Design: [README](../../../03-design/use-cases/renameTerms/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Term resolver | service | `crates/customise/src/terms.rs` | no (to build) |
| `/api/v1/scopes/{scope_id}/terms` | API endpoints | `docs/03-design/api/customise.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-212 to RN-222 | `crates/customise/tests/` | planned |
