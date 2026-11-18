---
id: UC-007
name: signIn
title: Sign in, or recover access
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-007 Sign in, or recover access — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/signIn/README.md) | [Analysis](../../../02-analysis/use-cases/signIn/README.md) | [Design](../../../03-design/use-cases/signIn/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/signIn/README.md#tests) |

Design: [README](../../../03-design/use-cases/signIn/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Sign-in pipeline | service | `crates/identity/src/sign_in.rs` | no (to build) |
| OIDC client | adapter | `crates/identity/src/oidc.rs` | no (to build) |
| `/api/v1/auth/sign-in`, `/second-factor`, `/oidc/*`, `/recovery` | API endpoints | `docs/03-design/api/identity.openapi.yaml` | no (to build) |
| `identity.session`, `identity.sign_in_attempt` | tables | `docs/03-design/data-model/identity.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-024, RN-027 to RN-031 | `crates/identity/tests/` | planned |
