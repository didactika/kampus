---
id: DATA-PROGRESS
status: proposed
origin: requirements
sourceOfTruth: model
---

# Data model: progress

PostgreSQL schema `progress`, owned by `kampus-progress`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [progress domain model](../../00-domain-model/progress.md). Names are logical; physically the schema carries the installation prefix, `kmp_progress` by default (DEC-039).

![Progress schema](./progress.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `rule` | `id`, `course_module_id` FK → `activities.course_module` (`CASCADE`), `kind` (`manual`, `view`, `submit`, `grade`, `pass`, `type_rule`), `type_rule_code`, `parameters` jsonb validated by the rule's schema | `UNIQUE` (`course_module_id`, `kind`, `type_rule_code`) |
| `activity_state` | `course_module_id` FK (`CASCADE`), `account_id` FK → `identity.account`, `state`, `source` (`automatic`, `manual`), `at`, `version` | PK (`course_module_id`, `account_id`); `CHECK` state in the five values (RN-115) |
| `activity_exception` | `course_module_id` FK (`CASCADE`), `account_id` FK, `state`, `reason`, `by`, `at` | PK (`course_module_id`, `account_id`) (RN-119) |
| `activity_history` | `id`, `course_module_id`, `account_id`, `from_state`, `to_state`, `cause`, `at` | Append-only |
| `criterion` | `id`, `course_id` FK → `courses.course` (`CASCADE`), `kind`, `aggregation` (`all`, `any`) | Leaves in typed tables below |
| `criterion_activity` | `criterion_id` FK (`CASCADE`), `course_module_id` FK (`RESTRICT`), `required_state` | PK (`criterion_id`, `course_module_id`) |
| `criterion_grade` | `criterion_id` PK FK (`CASCADE`), `record_id` FK → `records.record` (`RESTRICT`), `min_percent` | |
| `criterion_date` | `criterion_id` PK FK (`CASCADE`), `on_or_after` | |
| `criterion_duration` | `criterion_id` PK FK (`CASCADE`), `enrolled_for` interval | |
| `criterion_role_approval` | `criterion_id` FK (`CASCADE`), `role_id` FK → `access.role` (`RESTRICT`) | PK (`criterion_id`, `role_id`) |
| `criterion_course` | `criterion_id` FK (`CASCADE`), `course_id` FK → `courses.course` (`RESTRICT`) | PK (`criterion_id`, `course_id`) |
| `prerequisite` | `course_id` FK (`CASCADE`), `requires_course_id` FK (`CASCADE`) | PK (`course_id`, `requires_course_id`); `CHECK` differ; cycles refused on save (RN-122) |
| `course_state` | `course_id` FK (`CASCADE`), `account_id` FK, `state`, `completed_at`, `revoked_at`, `version` | PK (`course_id`, `account_id`) |
| `course_history` | `id`, `course_id`, `account_id`, `event` (`completed`, `revoked`), `at`, `cause` | Append-only |

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Criteria in one generic table with free integers and text | One typed table per criterion kind with foreign keys |
| States as unchecked integers, some never stored | Checked enumeration; display decided at read time |
| Recalculation that deletes every completion of the course | Jobs recompute derived states; people's records are kept |
| Completion written once and never re-checked | Re-evaluation by listeners; revocation recorded |
| Prerequisite cycles accepted | Cycle check on save |
