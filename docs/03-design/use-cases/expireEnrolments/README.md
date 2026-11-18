---
id: UC-020
name: expireEnrolments
title: Expire enrolments
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-020 Expire enrolments — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/expireEnrolments/README.md) | [Analysis](../../../02-analysis/use-cases/expireEnrolments/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/expireEnrolments/README.md) | [Tests](../../../04-implementation/use-cases/expireEnrolments/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/expireEnrolments/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Expiry job | job | `crates/enrol/src/jobs/expire.rs` |
| `enrol.expiry_outcome` | table | `docs/03-design/data-model/enrolment.md` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| enrol.enrolment | update or delete |
| enrol.expiry_outcome | insert |
| access.grant | delete owned grants |
