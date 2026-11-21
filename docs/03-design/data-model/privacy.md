---
id: DATA-PRIVACY
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: privacy

PostgreSQL schema `privacy`, owned by `kampus-privacy`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [privacy domain model](../../00-domain-model/privacy.md). Names are logical; physically the schema carries the installation prefix, `kmp_privacy` by default (DEC-039).

![privacy schema](./privacy.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `data_category` | `code` PK, `module`, `purpose`, `legal_basis` | Synchronised from privacy handlers (RN-178) |
| `request` | `id`, `account_id` FK → `identity.account`, `kind` (`export`, `deletion`), `state` (`pending`, `approved`, `running`, `held`, `completed`, `rejected`, `failed`), `requested_by` FK → `identity.account`, `reason`, `approved_by`, `due_by`, `created_at`, `completed_at` | Partial `UNIQUE` (`account_id`, `kind`) for open requests; index (`state`, `due_by`) |
| `request_item` | `request_id` FK (`CASCADE`), `module`, `state`, `job_id` FK → `tasks.job`, `error` | PK (`request_id`, `module`) |
| `retention_rule` | `category_code` FK → `data_category`, `campus_id`, `keep_for` interval, `action` (`erase`, `anonymise`) | PK (`category_code`, `campus_id`) |
| `legal_hold` | `id`, `account_id` FK (nullable), `scope_id` FK → `access.scope` (nullable), `reason`, `until`, `created_by` | `CHECK` exactly one of `account_id` and `scope_id` |
| `policy` | `id`, `code`, `version`, `text` jsonb, `required`, `published_at` | `UNIQUE` (`code`, `version`) |
| `acceptance` | `policy_id` FK, `account_id` FK (`CASCADE`), `accepted_at`, `on_behalf_of` FK → `identity.account` | PK (`policy_id`, `account_id`) |
| `export_package` | `request_id` FK (`CASCADE`), `file_id` FK → `files.file`, `expires_at` | PK (`request_id`, `file_id`) |

The first part of privacy (step 1) creates `request` (account, `requester` as a soft reference, kind, state `pending`, `completed`, `failed` or `expired`, `due_at`, `expires_at`) and `request_item` (one per data category, with the exported `data` as JSON until the package expires, the state and the error). Exports are answered by the handlers modules declare (`Module.privacy_handlers`: category, purpose, `export`); today records, progress, notify and calendar declare one each. Deletion, retention rules, legal holds, policies and acceptances, and packages as files arrive in later parts.

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Deletion that removes the account row and breaks every reference | The tombstone of RN-034 |
| Modules forgotten by the export | A handler declared per module, checked by the build |
| Retention expressed per screen | Declared categories with rules per campus |
