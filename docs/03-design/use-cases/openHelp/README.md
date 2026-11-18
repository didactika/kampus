---
id: UC-059
name: openHelp
title: Open help for the current screen
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-059 Open help for the current screen — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/openHelp/README.md) | [Analysis](../../../02-analysis/use-cases/openHelp/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/openHelp/README.md) | [Tests](../../../04-implementation/use-cases/openHelp/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/openHelp/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Help panel and resolver | UI | `web/src/shell/help/` |
| Help topics of the handbook | content | `handbook/` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| customise (local pages) | select |
