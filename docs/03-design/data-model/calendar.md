---
id: DATA-CALENDAR
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: calendar

PostgreSQL schema `calendar`, owned by `kampus-calendar`, following DEC-014, DEC-021 and DEC-022. Rules refer to the [calendar domain model](../../00-domain-model/calendar.md). Names are logical; physically the schema carries the installation prefix, `kmp_calendar` by default (DEC-039).

![Calendar schema](./calendar.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `entry` | `id`, `scope_id` FK → `access.scope` (`CASCADE`), `group_id` FK → `courses.group` (`CASCADE`, nullable), `owner_id` FK → `identity.account`, `title`, `description`, `starts_at`, `ends_at`, `all_day_date` date, `time_zone`, `rrule`, `version` | `CHECK` either instants or an all-day date; `CHECK` `ends_at` >= `starts_at`; index (`scope_id`, `starts_at`) |
| `entry_exception` | `entry_id` FK (`CASCADE`), `occurrence_at`, `cancelled` boolean, `override` jsonb (validated) | PK (`entry_id`, `occurrence_at`) |
| `activity_date` | `course_module_id` FK → `activities.course_module` (`CASCADE`), `kind` (`opens`, `due`, `closes`), `at` | PK (`course_module_id`, `kind`); refreshed by events (RN-157) |
| `subscription` | `id`, `scope_id` FK → `access.scope` (`CASCADE`), `url`, `refresh_every` interval, `last_refreshed_at`, `last_error` | |
| `imported_entry` | `subscription_id` FK (`CASCADE`), `uid`, `sequence`, `starts_at`, `ends_at`, `rrule`, `data` jsonb | PK (`subscription_id`, `uid`) |
| `feed_token` | `id`, `account_id` FK → `identity.account` (`CASCADE`), `hash` `UNIQUE`, `created_at`, `revoked_at` | |

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Level of an entry inferred from the first non-zero id | One scope reference |
| One row per occurrence of a repeating event | One rule, expanded when read |
| Imported events removed only when counts differ, without a trace | Diff by UID and sequence, every change recorded |
| Activity dates copied by every module | A projection refreshed by events |
