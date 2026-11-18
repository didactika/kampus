---
id: DATA-ACTIVITIES
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: activities

PostgreSQL schema `activities`, owned by `kampus-activities`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [activities domain model](../../00-domain-model/activities.md). Names are logical; physically the schema carries the installation prefix, `kmp_activities` by default (DEC-039). Each activity type owns its own schema (`activity_<type>`) for its instance data, with a foreign key to `course_module` and `ON DELETE CASCADE`.

![Activities schema](./activities.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `activity_type` | `code` PK, `kind` (`activity`, `material`), `contract_version`, `enabled`, `features` text[] checked | Synchronised from plugin manifests (DEC-015) |
| `course_module` | `id`, `scope_id` FK → `access.scope` `UNIQUE`, `section_id` FK → `courses.section` (`CASCADE`), `type_code` FK → `activity_type`, `title`, `sort_key`, `visibility`, `visible_from`, `visible_until`, `on_course_page`, `state` (`active`, `in_bin`, `failed_deletion`), `version` | `UNIQUE` (`section_id`, `sort_key`); `CHECK` `visible_until` > `visible_from`; index (`section_id`, `state`) |
| `module_setting` | `course_module_id` PK FK (`CASCADE`), `values` jsonb validated by the type's settings schema | Settings the type declares; common settings are columns of `course_module` |
| `condition` | `id`, `course_module_id` FK (`CASCADE`, nullable), `section_id` FK → `courses.section` (`CASCADE`, nullable), `parent_id` FK → `condition` (`CASCADE`), `operator` (`all`, `any`, `not`, `leaf`), `kind`, `show_when_unmet` | `CHECK` exactly one of `course_module_id`, `section_id`, `parent_id` set; leaves have one row in their kind's table |
| `condition_date` | `condition_id` PK FK (`CASCADE`), `from`, `until` | `CHECK` at least one set |
| `condition_progress` | `condition_id` PK FK (`CASCADE`), `course_module_id` FK → `course_module` (`RESTRICT`), `required_state` | |
| `condition_group` | `condition_id` PK FK (`CASCADE`), `group_id` FK → `courses.group` (`RESTRICT`, nullable for "any group"), `group_set_id` FK (`RESTRICT`, nullable) | |
| `condition_profile` | `condition_id` PK FK (`CASCADE`), `field_id` FK → `customise.field` (`RESTRICT`), `operator`, `value` | |
| `exception` | `id`, `course_module_id` FK (`CASCADE`), `account_id` FK → `identity.account` (`CASCADE`, nullable), `group_id` FK → `courses.group` (`CASCADE`, nullable), `opens_at`, `closes_at`, `due_at`, `cutoff_at`, `time_limit` interval, `attempts` integer, `reason`, `created_by`, `version` | `CHECK` exactly one of `account_id`, `group_id`; `UNIQUE` (`course_module_id`, `account_id`); `UNIQUE` (`course_module_id`, `group_id`); `CHECK` at least one value set |
| `bin_item` | `id`, `course_id`, `course_module_id` FK (`CASCADE`), `original_section_id`, `original_sort_key`, `deleted_by`, `deleted_at`, `expires_at`, `error` | One per binned course module |

Condition plugins (DEC-003) add their own leaf tables in their schema, with the same pattern; an unknown kind makes the course module unavailable (RN-082).

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Polymorphic `module` + `instance` pair | Each type's instance table references `course_module` |
| Availability as JSON in a text column with ids inside | Typed condition rows with foreign keys |
| Overrides in one table per activity, with different rules | One `exception` table for every activity type |
| Uniqueness of overrides checked in code | Unique constraints and a `CHECK` for person xor group |
| Deletion that leaves hidden orphans when it fails | Bin item with a visible failed state |
