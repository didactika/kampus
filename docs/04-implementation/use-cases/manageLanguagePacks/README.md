---
id: UC-046
name: manageLanguagePacks
title: Install language packs and customise messages
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-046 Install language packs and customise messages — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageLanguagePacks/README.md) | [Analysis](../../../02-analysis/use-cases/manageLanguagePacks/README.md) | [Design](../../../03-design/use-cases/manageLanguagePacks/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/manageLanguagePacks/README.md#tests) |

Design: [README](../../../03-design/use-cases/manageLanguagePacks/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Pack installer and customisations | service | `crates/i18n/src/packs.rs` | no (to build) |
| `/api/v1/language-packs`, `/api/v1/scopes/{scope_id}/messages` | API endpoints | `docs/03-design/api/i18n.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-187 to RN-194 | `crates/i18n/tests/` | planned |
