---
id: UC-040
name: subscribeCalendar
title: Subscribe to or export a calendar
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-040 Subscribe to or export a calendar — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/subscribeCalendar/README.md) | [Analysis](../../../02-analysis/use-cases/subscribeCalendar/README.md) | [Design](../../../03-design/use-cases/subscribeCalendar/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/subscribeCalendar/README.md#tests) |

Design: [README](../../../03-design/use-cases/subscribeCalendar/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Subscription refresher | job | `crates/calendar/src/subscriptions.rs` | no (to build) |
| `/api/v1/calendar-subscriptions`, `/api/v1/calendar-feeds` | API endpoints | `docs/03-design/api/calendar.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-159 to RN-161 | `crates/calendar/tests/` | planned |
