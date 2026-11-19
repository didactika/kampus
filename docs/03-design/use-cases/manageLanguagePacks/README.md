---
id: UC-046
name: manageLanguagePacks
title: Install language packs and customise messages
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-046 Install language packs and customise messages — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageLanguagePacks/README.md) | [Analysis](../../../02-analysis/use-cases/manageLanguagePacks/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/manageLanguagePacks/README.md) | [Tests](../../../04-implementation/use-cases/manageLanguagePacks/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/manageLanguagePacks/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Pack installer and customisations | service | `crates/i18n/src/packs.rs` |
| `/api/v1/i18n/language-packs`, `/api/v1/access/scopes/{scope_id}/messages` | API endpoints | `docs/03-design/api/i18n.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| i18n.pack, i18n.translation, i18n.customised_message | insert, update, delete |
