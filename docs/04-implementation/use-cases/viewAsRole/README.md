---
id: UC-005
name: viewAsRole
title: View a scope as another role
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-005 View a scope as another role — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/viewAsRole/README.md) | [Analysis](../../../02-analysis/use-cases/viewAsRole/README.md) | [Design](../../../03-design/use-cases/viewAsRole/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/viewAsRole/README.md#tests) |

Design: [README](../../../03-design/use-cases/viewAsRole/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| View-as service | service | `crates/access/src/view_as.rs` | no (to build) |
| `/api/v1/scopes/{scope_id}/view-as` | API endpoint | `docs/03-design/api/access.openapi.yaml` | no (to build) |
| `access.view_as_session` | table | `docs/03-design/data-model/access.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-018, RN-019 | `crates/access/tests/` | planned |
