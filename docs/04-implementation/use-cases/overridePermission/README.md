---
id: UC-002
name: overridePermission
title: Override permissions in a scope
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-002 Override permissions in a scope — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/overridePermission/README.md) | [Analysis](../../../02-analysis/use-cases/overridePermission/README.md) | [Design](../../../03-design/use-cases/overridePermission/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/overridePermission/README.md#tests) |

Design: [README](../../../03-design/use-cases/overridePermission/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Override service | service | `crates/access/src/overrides.rs` | no (to build) |
| `PUT /api/v1/scopes/{scope_id}/overrides` | API endpoint | `docs/03-design/api/access.openapi.yaml` | no (to build) |
| `access.permission_override` | table | `docs/03-design/data-model/access.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-004, RN-005, RN-008, RN-012, RN-014, RN-015 | `crates/access/tests/` | planned |
