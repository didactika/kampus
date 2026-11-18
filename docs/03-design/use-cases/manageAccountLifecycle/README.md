---
id: UC-009
name: manageAccountLifecycle
title: Manage the lifecycle of accounts
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-009 Manage the lifecycle of accounts — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/manageAccountLifecycle/README.md) | [Analysis](../../../02-analysis/use-cases/manageAccountLifecycle/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/manageAccountLifecycle/README.md) | [Tests](../../../04-implementation/use-cases/manageAccountLifecycle/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/manageAccountLifecycle/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Account lifecycle service | service | `crates/identity/src/service/lifecycle.rs` |
| Deletion job | job | `crates/identity/src/jobs/delete_account.rs` |
| `/api/v1/accounts/{account_id}/transitions`, bulk | API endpoints | `docs/03-design/api/identity.openapi.yaml` |
| `identity.account`, `identity.account_transition` | tables | `docs/03-design/data-model/identity.md` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| identity.account | update |
| identity.account_transition | insert |
| identity.session, identity.api_token | delete or revoke |
| every module | erase personal data on deletion |
