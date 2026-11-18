---
id: UC-049
name: installPlugin
title: Install a plugin
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-049 Install a plugin — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/installPlugin/README.md) | [Analysis](../../../02-analysis/use-cases/installPlugin/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/installPlugin/README.md) | [Tests](../../../04-implementation/use-cases/installPlugin/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/installPlugin/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Plugin installer | service | `crates/kampus-plugin-host/src/install.rs` |
| `/api/v1/plugins` | API endpoints | `docs/03-design/api/plugins.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| plugins.plugin, plugins.version, plugins.grant, plugins.installation, plugins.migration | insert, update |
| the plugin's own schema | create |
