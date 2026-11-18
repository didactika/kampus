---
id: UC-006
name: signUp
title: Sign up and confirm an account
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-006 Sign up and confirm an account — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/signUp/README.md) | [Analysis](../../../02-analysis/use-cases/signUp/README.md) | [Design](../../../03-design/use-cases/signUp/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/signUp/README.md#tests) |

Design: [README](../../../03-design/use-cases/signUp/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Account service | service | `crates/kampus-identity/src/account.rs` | no (to build) |
| `/api/v1/auth/sign-up`, `/api/v1/auth/confirm` | API endpoints | `docs/03-design/api/identity.openapi.yaml` | no (to build) |
| `identity.account`, `identity.one_time_token` | tables | `docs/03-design/data-model/identity.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-022 to RN-027 | `crates/kampus-identity/tests/` | planned |
