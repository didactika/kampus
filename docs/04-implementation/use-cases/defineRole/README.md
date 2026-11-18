---
id: UC-004
name: defineRole
title: Define a role from a template
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-004 Define a role from a template — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/defineRole/README.md) | [Analysis](../../../02-analysis/use-cases/defineRole/README.md) | [Design](../../../03-design/use-cases/defineRole/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/defineRole/README.md#tests) |

Design: [README](../../../03-design/use-cases/defineRole/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Role service | service | `crates/access/src/service/role.rs` | no (to build) |
| `/api/v1/campuses/{campus_id}/roles`, `/api/v1/roles/{role_id}`, reset preview and reset | API endpoints | `docs/03-design/api/access.openapi.yaml` | no (to build) |
| `access.role`, `access.role_permission`, `access.role_allowance` | tables | `docs/03-design/data-model/access.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-008, RN-015, RN-017 | `crates/access/tests/` | planned |
