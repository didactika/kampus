---
id: DATA-SETTINGS
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: settings

PostgreSQL schema `settings`, owned by `kampus-settings`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [settings domain model](../../00-domain-model/settings.md). Names are logical; physically the schema carries the installation prefix, `kmp_settings` by default (DEC-039).

![settings schema](./settings.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `definition` | `key` PK, `module`, `summary`, `kind` jsonb (type and bounds), `default_value` jsonb, `levels` jsonb, `permission`, `restart_required`, `retired_at` | Synchronised from declarations (DEC-015, RN-162); a key no module declares is retired and its values stay |
| `value` | `id`, `definition_key` FK → `definition` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `value` jsonb, `locked`, `version`, `updated_at` | `UNIQUE` (`definition_key`, `scope_id`); index on `scope_id`. Sensitive settings, encrypted with the installation key (RN-167), arrive with the first module that declares one |
| `change` | `id`, `definition_key` FK → `definition` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `actor` (the operator when absent; not a foreign key, it outlives the account), `before` jsonb, `after` jsonb, `locked`, `at` | Append-only; index (`definition_key`, `at`) |

Values fixed by the deployment are not stored: they are read from the configuration at start-up and returned with their source (RN-166).

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Untyped name and value text rows, validated by each screen | A declared schema checked on every write |
| Settings of plugins in one global table keyed by a concatenated name | Definitions per module, foreign keys from values |
| Secrets readable by anyone who can open the settings page | Encrypted, never returned |
| No record of who changed a setting | An append-only change table |
