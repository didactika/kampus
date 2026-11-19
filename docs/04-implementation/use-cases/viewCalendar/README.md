---
id: UC-039
name: viewCalendar
title: See the calendar
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-039 See the calendar — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/viewCalendar/README.md) | [Analysis](../../../02-analysis/use-cases/viewCalendar/README.md) | [Design](../../../03-design/use-cases/viewCalendar/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/viewCalendar/README.md#tests) |

Design: [README](../../../03-design/use-cases/viewCalendar/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Calendar view service | service | `crates/calendar/src/view.rs` | no (to build) |
| `/api/v1/calendar/calendar` | API endpoints | `docs/03-design/api/calendar.openapi.yaml` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-156 to RN-160 | `crates/calendar/tests/` | planned |
