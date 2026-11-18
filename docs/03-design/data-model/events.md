---
id: DATA-EVENTS
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: events and audit log

PostgreSQL schema `events`, owned by `kampus-events`, following DEC-014, DEC-016, DEC-021 and DEC-022. Rules refer to the [events domain model](../../00-domain-model/events.md). Names are logical; physically the schema carries the installation prefix, `kmp_events` by default (DEC-039).

![Events schema](./events.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `event_type` | `code` PK, `version` integer, `module`, `schema` jsonb, `audited` boolean, `personal_fields` text[] | Synchronised from declarations (DEC-015) |
| `event` | `id`, `type_code` FK → `event_type`, `type_version`, `scope_id`, `actor_id`, `object_id`, `payload` jsonb (validated by its schema), `occurred_at`, `request_id` | Index (`object_id`, `occurred_at`); partitioned by month; retained by policy |
| `outbox` | `event_id` PK FK → `event` (`CASCADE`), `created_at`, `dispatched_at` | Partial index on undispatched rows; claimed with `FOR UPDATE SKIP LOCKED` |
| `listener` | `id`, `module`, `type_code` FK → `event_type`, `handler`, `enabled` | `UNIQUE` (`module`, `type_code`, `handler`) |
| `delivery` | `listener_id` FK (`CASCADE`), `event_id` FK (`CASCADE`), `state` (`pending`, `done`, `failed`, `dead`), `attempts`, `next_attempt_at`, `last_error` | PK (`listener_id`, `event_id`); index on (`state`, `next_attempt_at`) |
| `audit_entry` | `id`, `event_id` FK → `event`, `campus_id`, `scope_id`, `actor_id`, `object_id`, `summary`, `at` | Append-only (no `UPDATE` grant); partitioned by month; index (`scope_id`, `at`) and (`actor_id`, `at`) |
| `retention_policy` | `campus_id`, `kind` (`audit`, `event_payload`), `keep_for` interval | PK (`campus_id`, `kind`) |

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Log rows buffered in memory and written after the request | Audit entries in the change's transaction |
| Serialised PHP data in log columns | Payloads validated by a declared JSON schema |
| Observer failures caught and printed as debugging | Deliveries with state, retries and a dead-letter list |
| Retention "never delete" by default | A retention policy per campus |
