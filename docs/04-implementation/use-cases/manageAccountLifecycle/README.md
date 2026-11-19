---
id: UC-009
name: manageAccountLifecycle
title: Manage the lifecycle of accounts
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-009 Manage the lifecycle of accounts — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageAccountLifecycle/README.md) | [Analysis](../../../02-analysis/use-cases/manageAccountLifecycle/README.md) | [Design](../../../03-design/use-cases/manageAccountLifecycle/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/manageAccountLifecycle/README.md#tests) |

Design: [README](../../../03-design/use-cases/manageAccountLifecycle/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Account lifecycle service | service | `crates/identity/src/service/lifecycle.rs` | no (to build) |
| Deletion job | job | `crates/identity/src/jobs/delete_account.rs` | no (to build) |
| `/api/v1/identity/accounts/{account_id}/transitions`, bulk | API endpoints | `docs/03-design/api/identity.openapi.yaml` | no (to build) |
| `identity.account`, `identity.account_transition` | tables | `docs/03-design/data-model/identity.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-023, RN-031, RN-034, RN-035, RN-038 | `crates/identity/tests/` | planned |
