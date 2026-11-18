---
id: UC-037
name: setNotificationPreferences
title: Choose how to be notified
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-037 Choose how to be notified — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/setNotificationPreferences/README.md) | [Analysis](../../../02-analysis/use-cases/setNotificationPreferences/README.md) | [Design](../../../03-design/use-cases/setNotificationPreferences/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/setNotificationPreferences/README.md#tests) |

Design: [README](../../../03-design/use-cases/setNotificationPreferences/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Preference resolver | service | `crates/notify/src/preferences.rs` | no (to build) |
| `/api/v1/notification-preferences` | API endpoint | `docs/03-design/api/notify.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-147 to RN-151 | `crates/notify/tests/` | planned |
