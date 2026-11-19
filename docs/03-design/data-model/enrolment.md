---
id: DATA-ENROL
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: enrolment

PostgreSQL schema `enrol`, owned by `kampus-enrol`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [enrolment domain model](../../00-domain-model/enrolment.md). Names are logical; physically the schema carries the installation prefix, `kmp_enrol` by default (DEC-039).

![Enrolment schema](./enrolment.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `instance` | `id`, `course_id` FK → `courses.course` (`CASCADE`), `method_code` (`manual`, `self`, `system_group` or a plugin's code), `name`, `role_id` FK → `access.role` (`RESTRICT`), `enabled`, `open_from`, `open_until`, `default_days`, `capacity`, `requires_approval`, `expiry_action`, `version` | `UNIQUE` (`course_id`, lower(`name`)); `CHECK` `open_until` > `open_from`; `CHECK` `capacity` and `default_days` > 0; `CHECK` `expiry_action` in (`suspend`, `remove`, `keep_without_role`); `method_code` a lower-case code. An instance limited to a section (`section_id`) arrives with access conditions (T-C.08) |
| `instance_setting` | `instance_id` PK FK (`CASCADE`), `values` jsonb validated by the method's declared schema | Only for settings a method declares; arrives with the first method plugin that declares settings |
| `self_key` | `id`, `instance_id` FK (`CASCADE`), `group_id` FK → `courses.group` (`CASCADE`, nullable), `hash` (Argon2id, as passwords) | `UNIQUE` (`instance_id`, `group_id`) `NULLS NOT DISTINCT` (RN-064) |
| `linked_group` | `instance_id` PK FK (`CASCADE`), `system_group_id` FK → `users.system_group` (`CASCADE`) | One source per instance (RN-067) |
| `linked_course` | `instance_id` PK FK (`CASCADE`), `source_course_id` FK → `courses.course` (`CASCADE`) | `CHECK` source differs from the instance's course; cycles refused by the service in a serialisable transaction (RN-067). Arrives with the course-link method |
| `enrolment` | `id`, `instance_id` FK (`CASCADE`), `account_id` FK → `identity.account` (`CASCADE`), `status`, `starts_at`, `ends_at`, `expired_at` (when the expiry rule handled it), `created_by`, `reason`, `version` | `UNIQUE` (`instance_id`, `account_id`) (RN-062); `CHECK` `ends_at` > `starts_at`; index (`account_id`, `status`) and (`instance_id`, `status`). `payment_ref` arrives with the payment port |
| `enrolment_reference` | External references of enrolments (step 2), in the shape of the [conventions](conventions.md#external-references); the campus is the campus of the course | Read with `enrol.enrolment.read`, set with `enrol.enrolment.manage`; a person is enrolled through a manual instance by one, and the same identifier never names the enrolment of someone else |
| `enrolment_grant` | `enrolment_id` FK (`CASCADE`), `grant_id` FK → `access.grant` (`CASCADE`) | PK (`enrolment_id`, `grant_id`); the service removes the grant when the enrolment goes or is suspended, and the grant covers the enrolment's period (RN-061) |
| `enrolment_group_member` | `instance_id` FK (`CASCADE`), `group_id` FK → `courses.group` (`CASCADE`), `account_id` FK → `identity.account` | PK (`instance_id`, `group_id`, `account_id`): the memberships an instance owns; the membership in `courses.group_member` has origin `enrol` and goes when no instance owns it any more (DEC-038) |
| `request` | `id`, `instance_id` FK (`CASCADE`), `account_id` FK, `message`, `group_id` (the group of the key given), `state`, `decided_by`, `decided_at`, `reason`, `created_at` | partial `UNIQUE` (`instance_id`, `account_id`) where state is `pending` |
| `expiry_outcome` | `id`, `enrolment_id` FK (`SET NULL`), `instance_id`, `account_id`, `action`, `at` | Append-only (RN-068); kept when the enrolment is removed |
| `self_attempt` | `id`, `instance_id`, `account_id`, `address`, `succeeded`, `at` | Throttling of key attempts: five wrong keys an hour per person and instance (RN-064); purged after retention |

## Active participation (RN-060)

One query answers "is this person an active participant of this course": an `EXISTS` over `enrolment` joined to `instance` with `status = 'active'`, `enabled`, and `now()` (the database clock) within the period. Its result is cached per person and course for the request and invalidated by `enrol.*` events.

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Generic integer and text columns reused by every method for different meanings | Typed columns and typed link tables; a validated settings document only for what a method declares |
| Keys in clear text compared as strings, with hints | Hashes, constant-time comparison, no hints, throttling |
| Two sentinels for "forever" | `NULL` for open periods |
| Capacity checked by counting before inserting | Lock on the instance in the creating transaction |
| Roles of manual and self enrolments without an owner | Every enrolment owns its grants through `enrolment_grant` |
| Grades deleted when the last enrolment ends | Records and progress are never deleted by enrolment changes (RN-069) |
