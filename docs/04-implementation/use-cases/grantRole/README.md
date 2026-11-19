---
id: UC-001
name: grantRole
title: Grant a role in a scope
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-001 Grant a role in a scope — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/grantRole/README.md) | [Analysis](../../../02-analysis/use-cases/grantRole/README.md) | [Design](../../../03-design/use-cases/grantRole/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/grantRole/README.md#tests) |

Design: [README](../../../03-design/use-cases/grantRole/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Grant service | service | `crates/access/src/service/grant.rs` | no (to build) |
| `POST /api/v1/access/scopes/{scope_id}/grants` | API endpoint | `docs/03-design/api/access.openapi.yaml` | no (to build) |
| `access.grant` | table | `docs/03-design/data-model/access.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-003, RN-013, RN-015, RN-016 | `crates/access/tests/` | planned |
