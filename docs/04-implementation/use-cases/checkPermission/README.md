---
id: UC-003
name: checkPermission
title: Check a permission, with its explanation
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-003 Check a permission, with its explanation — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/checkPermission/README.md) | [Analysis](../../../02-analysis/use-cases/checkPermission/README.md) | [Design](../../../03-design/use-cases/checkPermission/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/checkPermission/README.md#tests) |

Design: [README](../../../03-design/use-cases/checkPermission/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Permission checker | service | `crates/access/src/service/check.rs` | no (to build) |
| Per-user access cache | cache | `crates/access/src/service/cache.rs` | no (to build) |
| `POST /api/v1/access/scopes/{scope_id}/checks` | API endpoint | `docs/03-design/api/access.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-003 to RN-011, RN-019, RN-021 | `crates/access/tests/` | planned |
