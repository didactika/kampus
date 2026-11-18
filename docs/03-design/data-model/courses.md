---
id: DATA-COURSES
status: proposed
origin: requirements
sourceOfTruth: model
---

# Data model: courses

PostgreSQL schema `courses`, owned by `kampus-courses`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [courses domain model](../../00-domain-model/courses.md). Table names are logical; physically every name carries the installation prefix, `kmp_` by default (DEC-039).

![Courses schema](./courses.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `category` | `id`, `scope_id` FK → `access.scope` `UNIQUE`, `campus_id` FK → `access.scope`, `parent_id` FK → `category` (`RESTRICT`), `code`, `name`, `description`, `sort_key` text, `visibility`, `visible_from`, `visible_until`, `version` | partial `UNIQUE` (`campus_id`, lower(`code`)) where code is set; `UNIQUE` (`parent_id`, `sort_key`); index on `parent_id` |
| `course` | `id`, `scope_id` FK `UNIQUE`, `campus_id`, `category_id` FK → `category` (`RESTRICT`), `short_name`, `full_name`, `code`, `summary`, `layout_code`, `starts_at`, `ends_at`, `state`, `state_changed_at`, `visibility`, `visible_from`, `visible_until`, `sort_key`, `template_id`, `version` | `UNIQUE` (`campus_id`, lower(`short_name`)); partial `UNIQUE` (`campus_id`, lower(`code`)); `CHECK` `ends_at` > `starts_at`; `CHECK` state in four values; `UNIQUE` (`category_id`, `sort_key`) |
| `course_layout_setting` | `course_id` FK (`CASCADE`), `layout_code`, `values` jsonb validated by the layout's declared schema | PK (`course_id`, `layout_code`); keeps the settings of each layout used, so switching back restores them (RN-052) |
| `course_owner` | `course_id` FK (`CASCADE`), `account_id` FK → `identity.account`, `since` | PK (`course_id`, `account_id`) |
| `section` | `id`, `scope_id` FK `UNIQUE`, `course_id` FK (`CASCADE`), `parent_id` FK → `section` (`CASCADE`), `title`, `summary`, `sort_key`, `visibility`, `visible_from`, `visible_until`, `deleted_at`, `version` | `UNIQUE` (`course_id`, `parent_id`, `sort_key`) (`NULLS NOT DISTINCT`); trigger: parent in the same course and no cycle; index (`course_id`, `parent_id`) |
| `group` | `id`, `course_id` FK (`CASCADE`), `section_id` FK → `section` (`CASCADE`, nullable), `name`, `code`, `description`, `visibility`, `joining_key_hash`, `version` | `UNIQUE` (`course_id`, lower(`name`)); partial `UNIQUE` (`course_id`, lower(`code`)); `CHECK` visibility in four values |
| `group_member` | `group_id` FK → `group` (`CASCADE`), `account_id` FK → `identity.account`, `origin` (`manual`, `module`), `origin_module`, `added_at` | PK (`group_id`, `account_id`, `origin`); manual or module, never both in one row (RN-054) |
| `group_set` | `id`, `course_id` FK (`CASCADE`), `name`, `code` | `UNIQUE` (`course_id`, lower(`name`)) |
| `group_set_group` | `set_id` FK (`CASCADE`), `group_id` FK (`CASCADE`) | PK (`set_id`, `group_id`) (RN-058) |
| `course_template` | `id`, `scope_id` FK → `access.scope` (campus or category), `code`, `layout_code`, `sections` jsonb (validated outline), `settings` jsonb (validated) | `UNIQUE` (`scope_id`, `code`) |

Course modules are placed in sections by `kampus-activities`, which owns `activities.course_module (section_id FK → courses.section, sort_key)`; this schema never lists module identifiers (no list in a text column). Group mode is a setting of `kampus-settings`, resolved by scope (RN-057), not a column here.

## Effective visibility (RN-043)

`visibility` is `shown`, `hidden` or `scheduled` (with `visible_from` and `visible_until`). The effective visibility of a node is computed from the node and its ancestors in the scope closure: one query joins the ancestors and requires every one of them to be visible at `now()`. It is cached per scope subtree and invalidated by the `*_updated` and `*_moved` events. Nothing is ever written to the descendants.

## Ordering (RN-045, RN-046)

`sort_key` is a fractional index: a short text that sorts between its neighbours, so inserting or moving one node writes only that node. Keys are rebalanced, rarely, by a job that rewrites the keys of one parent in one transaction.

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Section position used as identity, renumbered on every move | Stable ids and fractional sort keys |
| List of module ids in a text column | Course modules reference their section |
| Visibility copied down with a one-level undo | Own visibility per node, effective visibility computed |
| Global sort order with limits per category | Order local to each parent |
| Format options as name and text value rows with a sentinel for "the course" | Typed layout settings validated by the layout's schema, per course |
| Group names unique only in forms; duplicate links in group sets | Unique constraints |
| Deletion right derived from the event log | Course owners as data, with a permission |
