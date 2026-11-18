---
id: UC-045
name: chooseLanguage
title: Choose a language
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-045 Choose a language — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/chooseLanguage/README.md) | [Analysis](../../../02-analysis/use-cases/chooseLanguage/README.md) | [Design](../../../03-design/use-cases/chooseLanguage/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/chooseLanguage/README.md#tests) |

Design: [README](../../../03-design/use-cases/chooseLanguage/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Language resolver and bundles | service | `crates/i18n/src/resolve.rs` | no (to build) |
| `/api/v1/me/language`, `/api/v1/languages/{code}/bundles/{module}` | API endpoints | `docs/03-design/api/i18n.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-189 to RN-192 | `crates/i18n/tests/` | planned |
