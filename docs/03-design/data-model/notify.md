---
id: DATA-NOTIFY
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: notifications and messaging

PostgreSQL schema `notify`, owned by `kampus-notify`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [notifications domain model](../../00-domain-model/notify.md). Names are logical; physically the schema carries the installation prefix, `kmp_notify` by default (DEC-039).

![Notifications schema](./notify.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `notification_type` | `code` PK, `module`, `required` boolean, `default_channels` text[] | Synchronised from declarations |
| `channel` | `code` PK, `enabled` | inbox, email, push, plugins |
| `preference` | `account_id` FK → `identity.account` (`CASCADE`), `type_code` FK → `notification_type`, `channel_code` FK → `channel`, `enabled`, `digest` (`none`, `daily`, `weekly`) | PK (`account_id`, `type_code`, `channel_code`) |
| `campus_default` | `campus_id`, `type_code` FK, `channel_code` FK, `enabled`, `locked` | PK (`campus_id`, `type_code`, `channel_code`) |
| `template` | `id`, `type_code` FK, `channel_code` FK, `locale`, `scope_id` FK → `access.scope`, `body` | `UNIQUE` (`type_code`, `channel_code`, `locale`, `scope_id`) |
| `notification` | `id`, `type_code` FK, `recipient_id` FK → `identity.account` (`CASCADE`), `object_id`, `payload` jsonb (validated), `created_at`, `read_at` | Index (`recipient_id`, `read_at`, `created_at`) |
| `delivery` | `notification_id` FK (`CASCADE`), `channel_code` FK, `state`, `attempts`, `next_attempt_at`, `last_error` | PK (`notification_id`, `channel_code`) |
| `conversation` | `id`, `kind` (`direct`, `group`, `course_group`), `course_group_id` FK → `courses.group` (`CASCADE`, nullable), `direct_key` text | `UNIQUE` (`direct_key`) for direct ones (ordered pair of account ids); `UNIQUE` (`course_group_id`) |
| `member` | `conversation_id` FK (`CASCADE`), `account_id` FK, `joined_at`, `left_at`, `muted`, `last_read_message_id` | PK (`conversation_id`, `account_id`) |
| `message` | `id`, `conversation_id` FK (`CASCADE`), `author_id` FK, `body`, `sent_at` | Index (`conversation_id`, `sent_at`) |
| `message_deletion` | `message_id` FK (`CASCADE`), `account_id` FK, `at` | PK (`message_id`, `account_id`); a job purges messages deleted by every member (RN-154) |
| `block` | `account_id` FK (`CASCADE`), `blocked_id` FK (`CASCADE`) | PK (`account_id`, `blocked_id`) |

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Preferences as concatenated names with comma lists of processors | Typed preference rows per type and channel |
| Mail sent inside the sender's request | One delivery job per recipient and channel |
| One-to-one conversations de-duplicated by read-then-insert on a non-unique hash | Unique key on the ordered pair |
| Messages kept forever after every member deleted them | Purge job and retention policy |
