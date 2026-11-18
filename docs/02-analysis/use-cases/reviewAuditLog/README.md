---
id: UC-036
name: reviewAuditLog
title: Review the audit log
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-036 Review the audit log — analysis realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/reviewAuditLog/README.md) | **Analysis** | [Design](../../../03-design/use-cases/reviewAuditLog/README.md) | [Implementation](../../../04-implementation/use-cases/reviewAuditLog/README.md) | [Tests](../../../04-implementation/use-cases/reviewAuditLog/README.md#tests) |

Requirements: [README](../../../01-requirements/use-cases/reviewAuditLog/README.md)

## Participating classes

| Class | Stereotype | Responsibility |
| --- | --- | --- |
| Audit screen, `GET /audit` | boundary | Collect the request |
| Audit query service | control | Apply RN-143 to RN-146 |
| Audit entry | entity | Persisted state |

## Collaboration

![Collaboration](./collaboration.svg)

## Notes

Every change is one transaction with its events in the outbox (DEC-016).
