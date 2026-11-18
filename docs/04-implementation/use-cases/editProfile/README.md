---
id: UC-010
name: editProfile
title: Edit a profile
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-010 Edit a profile — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/editProfile/README.md) | [Analysis](../../../02-analysis/use-cases/editProfile/README.md) | [Design](../../../03-design/use-cases/editProfile/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/editProfile/README.md#tests) |

Design: [README](../../../03-design/use-cases/editProfile/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Profile service | service | `crates/kampus-users/src/profile.rs` | no (to build) |
| `/api/v1/me/profile`, `/api/v1/accounts/{account_id}/profile` | API endpoints | `docs/03-design/api/identity.openapi.yaml` | no (to build) |
| `users.profile`, `users.profile_field_value` | tables | `docs/03-design/data-model/identity.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-032, RN-036 | `crates/kampus-users/tests/` | planned |
