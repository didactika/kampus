---
id: UC-038
name: sendMessage
title: Send a message
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-038 Send a message — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/sendMessage/README.md) | [Analysis](../../../02-analysis/use-cases/sendMessage/README.md) | [Design](../../../03-design/use-cases/sendMessage/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/sendMessage/README.md#tests) |

Design: [README](../../../03-design/use-cases/sendMessage/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Messaging service | service | `crates/kampus-notify/src/messaging.rs` | no (to build) |
| `/api/v1/conversations` | API endpoints | `docs/03-design/api/notify.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-152 to RN-155 | `crates/kampus-notify/tests/` | planned |
