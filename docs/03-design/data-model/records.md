---
id: DATA-RECORDS
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: academic records

PostgreSQL schema `records`, owned by `kampus-records`, following DEC-014, DEC-021, DEC-022, DEC-023 and DEC-026. Rules refer to the [records domain model](../../00-domain-model/records.md). Names are logical; physically the schema carries the installation prefix, `kmp_records` by default (DEC-039).

![Records schema](./records.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `category` | `id`, `course_id` FK → `courses.course` (`CASCADE`), `parent_id` FK → `category` (`CASCADE`), `root_course_id` FK → `courses.course` (`CASCADE`, set only on the root), `name`, `method`, `drop_lowest`, `keep_highest`, `exclude_empty`, `sort_key` (`C` collation), `version` | `CHECK` method in the eight methods; `CHECK` drop and keep not negative (the service refuses both set); one root per course (`UNIQUE` (`root_course_id`), which is null for every other category); `UNIQUE` (`parent_id`, `sort_key`); created with the first change of a course, with the total of the course |
| `record` | `id`, `course_id` FK (`CASCADE`), `kind` (`activity`, `manual`, `category_total`, `course_total`), `title`, `course_module_id` FK → `activities.course_module` (`CASCADE`, nullable), `item_number`, `category_id` FK → `category` (`CASCADE`, the category that aggregates it; null for the course total), `total_of_id` FK → `category` (`CASCADE`, the category a total totals), `grade_type`, `min`, `max`, `pass_grade`, `weight` numeric(9,5), `extra_credit`, `multiplier` numeric(9,5), `offset_value`, `hidden_until`, `locked_at`, `sort_key` (`C` collation), `version`; `scale_id` arrives with scales | `UNIQUE` (`course_module_id`, `item_number`) makes an activity declare each of its grades once; `UNIQUE` (`total_of_id`) gives each category one total; `UNIQUE` (`category_id`, `sort_key`); `CHECK` `max` > `min` |
| `entry` | `id`, `record_id` FK (`CASCADE`), `account_id` FK → `identity.account` (`CASCADE`), `raw` numeric(12,5), `text`, `feedback`, `source` (`activity`, `teacher`, `import`, `formula`), `actor` (soft reference to the account that made it), `at` | Append-only; index (`record_id`, `account_id`, `at`) |
| `grade` | `record_id` FK (`CASCADE`), `account_id` FK → `identity.account` (`CASCADE`), `raw`, `final` numeric(12,5), `text`, `overridden`, `excluded`, `hidden_until`, `locked_at`, `version`, `updated_at` | PK (`record_id`, `account_id`) — the upsert target (RN-096); the same grade sent again does not touch the row, so its version only moves with a real change |
| `scale` | `id`, `scope_id` FK → `access.scope`, `code`, `version`, `items` text[] | `UNIQUE` (`scope_id`, `code`, `version`) |
| `letter_set` | `id`, `scope_id` FK, `version` | `UNIQUE` (`scope_id`, `version`) |
| `letter_band` | `letter_set_id` FK (`CASCADE`), `min_percent` numeric(9,5), `letter` | PK (`letter_set_id`, `min_percent`) |
| `formula` | `record_id` PK FK (`CASCADE`), `expression` | Parsed and validated on save |
| `formula_dependency` | `record_id` FK → `formula` (`CASCADE`), `depends_on_id` FK → `record` (`RESTRICT`) | PK (`record_id`, `depends_on_id`); cycles refused on save (RN-107) |
| `recalculation` | `course_id` FK → `courses.course` (`CASCADE`), `account_id` FK → `identity.account` (`CASCADE`), `queued_at` | PK (`course_id`, `account_id`); the queue of people whose totals are recalculated (RN-099): a change inserts the people it touches with `ON CONFLICT DO NOTHING`, and the job `records.recalculate` takes a batch with `FOR UPDATE SKIP LOCKED`, recomputes them from the deepest category up in the same transaction and deletes the rows it took, so a failure leaves them queued |
| `grade_condition` | `condition_id` PK FK → `activities.condition` (`CASCADE`), `record_id` FK → `record` (`RESTRICT`), `min_percent`, `max_percent` | The grade condition type is declared by records, which owns its leaf table (DEC-045, DEC-079); RN-081: deleting the record is refused while the condition exists |

The categories, grade records, entries and grades (step 1) the recalculation queue (step 2), and scales and letter sets (step 3) exist; formulas and the grade condition arrive with the next parts of records. A scale record takes the range of its scale, from the first item (1) to the last, and grades with the position of an item; a record keeps the version of the scale it was set up with, and a scale or a letter set is available in its scope and below, the nearest definition winning. A total is a record whose grades are derived: the job computes each category from its children with the aggregation of its method (mean, weighted mean, simple weighted mean, natural, median, lowest, highest, mode), a natural category gives its total the range its children add up to, a total can be overridden, and a total that adds up something hidden is hidden from whoever may not see it. Grades survive the end of a participation (RN-105) once records depends on enrolment; today a person sees their own shown grades while they hold `records.view_own` in the course.

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Totals cached and recomputed by the first reader | Derived by a job; reads have no side effects |
| One coefficient column whose meaning changes with the parent's method | Typed `weight` and `extra_credit` columns validated per method |
| History written only for some changes; dates reused with other meanings | Append-only entries with source, actor and time |
| Scale items as a comma list | Versioned scales with an array of items |
| Uniqueness of activity records checked in code | Unique constraints per course module and item |
