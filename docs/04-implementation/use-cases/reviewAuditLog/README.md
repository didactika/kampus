---
id: UC-036
name: reviewAuditLog
title: Review the audit log
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-036 Review the audit log — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/reviewAuditLog/README.md) | [Analysis](../../../02-analysis/use-cases/reviewAuditLog/README.md) | [Design](../../../03-design/use-cases/reviewAuditLog/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/reviewAuditLog/README.md#tests) |

Design: [README](../../../03-design/use-cases/reviewAuditLog/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Audit query service | service | `crates/events/src/audit.rs` | no (to build) |
| `/api/v1/events/audit` | API endpoints | `docs/03-design/api/events.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-143 to RN-146 | `crates/events/tests/` | planned |
