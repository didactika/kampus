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
| `entry` | `id`, `scope_id` FK → `access.scope` (`CASCADE`, nullable), `owner_id` FK → `identity.account` (`CASCADE`, nullable), `author` (soft reference to the account that created it), `title`, `description`, `starts_at`, `ends_at`, `time_zone` (UTC by default), `all_day`, `recurrence` (RFC 5545 rule), `group_id` FK → `courses.group` (`CASCADE`, nullable), `version`, `created_at` | `CHECK` exactly one of `scope_id` and `owner_id`; indexes (`scope_id`, `starts_at`) and (`owner_id`, `starts_at`); a group entry is seen by the members of the group and whoever manages the calendar |
| `entry_exception` | `entry_id` FK (`CASCADE`), `occurrence_at`, `cancelled`, `title`, `starts_at`, `ends_at` | PK (`entry_id`, `occurrence_at`); `CHECK` `ends_at` after `starts_at`; one change per occurrence (step 2) |
| `activity_date` | `course_module_id` FK → `activities.course_module` (`CASCADE`), `kind` (`opens`, `due`, `closes`), `at` | PK (`course_module_id`, `kind`); refreshed by events (RN-157) |
| `subscription` | `id`, `scope_id` FK → `access.scope` (`CASCADE`), `url`, `title`, `refresh_minutes` (15 to 10080, 360 by default), `created_by` (soft), `created_at`, `last_refreshed_at`, `last_error` | `UNIQUE` (`scope_id`, `url`); read by the job `calendar.refresh` (step 3) |
| `imported_entry` | `subscription_id` FK (`CASCADE`), `uid`, `sequence`, `title`, `description`, `starts_at`, `ends_at`, `all_day`, `recurrence` | PK (`subscription_id`, `uid`); `CHECK` `ends_at` after `starts_at`; replaced as a diff on each read |
| `feed_token` | `id`, `account_id` FK → `identity.account` (`CASCADE`), `hash` (SHA-256 of the secret), `name`, `created_at`, `last_used_at`, `revoked_at` | `UNIQUE` (`hash`); index (`account_id`); the secret is shown once |

The `entry` table (step 1) exists. The dates of activities are not stored: the calendar of a course reads them from the activities each time and applies the exceptions of the person who asks (RN-157), so there is nothing to refresh. Step 2 adds recurrence, time zones, all-day and group entries, and the exceptions of occurrences: a recurring entry is stored once and expanded when a window is read, in the time zone of the entry so an occurrence keeps its time of day across daylight saving changes (`support/recurrence.rs`: `FREQ` daily, weekly, monthly or yearly with `INTERVAL`, `COUNT` or `UNTIL`, `BYDAY` for weekly and `BYMONTHDAY` for monthly rules, at most 1000 occurrences); changing one occurrence adds an exception, never a copy (RN-158). Step 3 adds personal feeds and subscriptions (RN-159, RN-161): a feed is a secret address (only its hash is kept) that gives, as iCalendar, what its owner sees from a month ago to almost a year ahead; a subscription reads the calendar of another system over http or https, only at public network addresses and without following redirects, at most 5 MB and 5000 events, matches its events by UID and applies what changed (`support/ical.rs` reads UTC, local times of a named time zone and whole days, and leaves cancelled events out).

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Level of an entry inferred from the first non-zero id | One scope reference |
| One row per occurrence of a repeating event | One rule, expanded when read |
| Imported events removed only when counts differ, without a trace | Diff by UID and sequence, every change recorded |
| Activity dates copied by every module | A projection refreshed by events |
