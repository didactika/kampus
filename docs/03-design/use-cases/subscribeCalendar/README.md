---
id: UC-040
name: subscribeCalendar
title: Subscribe to or export a calendar
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-040 Subscribe to or export a calendar — design realization

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/subscribeCalendar/README.md) | [Analysis](../../../02-analysis/use-cases/subscribeCalendar/README.md) | **Design** | [Implementation](../../../04-implementation/use-cases/subscribeCalendar/README.md) | [Tests](../../../04-implementation/use-cases/subscribeCalendar/README.md#tests) |

Analysis: [README](../../../02-analysis/use-cases/subscribeCalendar/README.md)

## Elements

| Design element | Kind | Code target |
| --- | --- | --- |
| Subscription refresher | job | `crates/calendar/src/subscriptions.rs` |
| `/api/v1/calendar-subscriptions`, `/api/v1/calendar-feeds` | API endpoints | `docs/03-design/api/calendar.openapi.yaml` |

## Sequence

![Sequence](./sequence.svg)

## Data touched

| Table or entity | Operation |
| --- | --- |
| calendar.subscription, calendar.imported_entry | insert, update, delete |
| calendar.feed_token | insert, update |
