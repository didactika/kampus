---
id: UC-059
name: openHelp
title: Open help for the current screen
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-059 Open help for the current screen — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/openHelp/README.md) | **Analysis** | [Design](../../../03-design/use-cases/openHelp/README.md) | [Implementation](../../../04-implementation/use-cases/openHelp/README.md) | [Tests](../../../04-implementation/use-cases/openHelp/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/openHelp/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Help panel of the shell | boundary | Collect the request |
| Help resolver | control | Apply HB-01 to HB-04 |
| Help topic | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Help content is static per release; progress changes are one transaction with their events (DEC-016).
