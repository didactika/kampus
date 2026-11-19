---
id: UC-034
name: downloadFile
title: Open or download a file
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-034 Open or download a file — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/downloadFile/README.md) | [Analysis](../../../02-analysis/use-cases/downloadFile/README.md) | [Design](../../../03-design/use-cases/downloadFile/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/downloadFile/README.md#tests) |

Design: [README](../../../03-design/use-cases/downloadFile/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| File access service | service | `crates/files/src/access.rs` | no (to build) |
| `/api/v1/files/files/{file_id}/url` | API endpoint | `docs/03-design/api/files.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-128 to RN-135 | `crates/files/tests/` | planned |
