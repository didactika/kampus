---
id: UC-045
name: chooseLanguage
title: Choose a language
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-045 Choose a language — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/chooseLanguage/README.md) | [Analysis](../../../02-analysis/use-cases/chooseLanguage/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/chooseLanguage/README.md) | [Tests](../../../04-implementation/use-cases/chooseLanguage/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/chooseLanguage/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Language resolver and bundles | service | `crates/i18n/src/resolve.rs` |
| `/api/v1/me/language`, `/api/v1/languages/{code}/bundles/{module}` | API endpoints | `docs/03-design/api/i18n.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| users.profile (language_code) | update |
| i18n.translation, i18n.customised_message | select |
