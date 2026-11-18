---
id: UC-058
name: shareCustomisation
title: Export and import customisations
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-058 Export and import customisations — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/shareCustomisation/README.md) | [Analysis](../../../02-analysis/use-cases/shareCustomisation/README.md) | [Design](../../../03-design/use-cases/shareCustomisation/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/shareCustomisation/README.md#tests) |

Design: [README](../../../03-design/use-cases/shareCustomisation/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Package service | service | `crates/kampus-customise/src/packages.rs` | no (to build) |
| `/api/v1/scopes/{scope_id}/customisation-packages`, `/customisation-imports` | API endpoints | `docs/03-design/api/customise.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-221 to RN-222 | `crates/kampus-customise/tests/` | planned |
