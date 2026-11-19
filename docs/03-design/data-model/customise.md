---
id: DATA-CUSTOMISE
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: customisation

PostgreSQL schema `customise`, owned by `kampus-customise`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [customisation domain model](../../00-domain-model/customise.md). Names are logical; physically the schema carries the installation prefix, `kmp_customise` by default (DEC-039).

![customisation schema](./customise.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `term` | `code` PK, `module`, `forms` jsonb (singular and plural per language), `retired_at` | Synchronised from declarations (RN-212); a term no module declares is retired and its overrides stay |
| `term_override` | `term_code` FK → `term` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `language_code`, `singular`, `plural`, `locked`, `version` | PK (`term_code`, `scope_id`, `language_code`); the foreign key to `i18n.language` arrives with languages |
| `field_kind` | `code` PK, `module`, `summary`, `retired_at` | The kinds of entity that accept custom fields, synchronised from declarations |
| `field` | `id`, `kind_code` FK → `field_kind` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `code`, `name`, `type` (`text`, `integer`, `date`, `boolean`, `choice`, `choices`, `account`), `required`, `visibility` (`public`, `private`), `max_length`, `sort_key` (`C` collation), `version` | `UNIQUE` (`kind_code`, `scope_id`, `code`); applies in its scope and below, and the nearest field with a code wins |
| `field_choice` | `id`, `field_id` FK (`CASCADE`), `code`, `label`, `sort_key` (`C` collation) | `UNIQUE` (`field_id`, `code`) |
| `field_value` | `id`, `field_id` FK (`CASCADE`), `text_value`, `integer_value` bigint, `date_value` date, `boolean_value`, `choice_id` FK → `field_choice` (`CASCADE`), `account_id` FK → `identity.account` (`CASCADE`), `updated_at` | `CHECK` at most one typed column is set (a multiple-choice value sets none and keeps its choices in `field_value_choice`); owned through the owning module's link table (`<module>.<entity>_field`: entity FK with `CASCADE`, `field_value_id` FK with `CASCADE`, PK (entity, field)), so there is no polymorphic owner (RN-214). Files arrive with `files` |
| `field_value_choice` | `field_value_id` FK → `field_value` (`CASCADE`), `choice_id` FK → `field_choice` (`CASCADE`) | PK (`field_value_id`, `choice_id`) |
| `page_kind` | `code` PK, `module`, `summary`, `regions` jsonb (in order), `retired_at` | Synchronised from declarations |
| `widget` | `code` PK, `module`, `summary`, `page_kinds` jsonb (every page kind when empty), `settings` jsonb (key, stored kind and default of each), `retired_at` | Synchronised from declarations; a retired widget is skipped when a layout is read (RN-216) |
| `menu_kind` | `code` PK, `module`, `summary`, `retired_at` | Synchronised from declarations |
| `layout` | `id`, `page_kind_code` FK → `page_kind` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `locked`, `version` | `UNIQUE` (`page_kind_code`, `scope_id`); the layout of the nearest scope applies whole, unless a scope above locks its own |
| `placement` | `id`, `layout_id` FK (`CASCADE`), `region`, `widget_code` FK → `widget` (`CASCADE`), `settings` jsonb (validated against the widget's settings), `permission` (who sees it; everyone when null), `position` | `UNIQUE` (`layout_id`, `region`, `position`); replaced with the whole layout, so a position is enough |
| `menu` | `id`, `menu_kind_code` FK → `menu_kind` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `locked`, `version` | `UNIQUE` (`menu_kind_code`, `scope_id`) |
| `menu_item` | `id`, `menu_id` FK (`CASCADE`), `parent_id` FK → `menu_item` (`CASCADE`), `label`, `target` (a path of the site or an `http`/`https` address), `permission`, `position` | Index (`menu_id`, `parent_id`, `position`); at most 100 items, 3 levels deep; a hidden item hides its children. Labels per language arrive with languages |
| `theme` | `id`, `scope_id` FK → `access.scope` (`CASCADE`), `tokens` jsonb (the tokens this scope sets, validated, RN-217), `locked`, `version` | `UNIQUE` (`scope_id`); tokens merge from the platform down to the scope read, stopping at the first scope that locks; the parent of a theme is the theme of the scope above, so it needs no column |
| `rule` | `id`, `scope_id` FK → `access.scope` (`CASCADE`), `owner_id` FK → `identity.account`, `name`, `trigger` (event type), `conditions` jsonb (typed tree), `actions` jsonb (typed list), `enabled`, `version` | Index (`trigger`) where `enabled` |
| `rule_run` | `id`, `rule_id` FK (`CASCADE`), `event_id` FK → `events.event`, `outcome`, `error`, `at` | Index (`rule_id`, `at`); purged by retention |
| `flag` | `code` PK, `module`, `summary`, `default_value` boolean, `retired_at` | Synchronised from declarations |
| `flag_value` | `flag_code` FK (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `enabled`, `locked` | PK (`flag_code`, `scope_id`) |
| `change` | `id`, `kind` (`term`, `flag`, `field`, `layout`, `menu`, `theme`), `object`, `scope_id` FK → `access.scope` (`CASCADE`), `actor` (not a foreign key, it outlives the account), `before` jsonb, `after` jsonb, `at` | Append-only (RN-222); index (`scope_id`, `at`); every change also publishes `customise.changed`. The package checksum arrives with packages |

Rules and packages arrive in later parts of customisation; until then their tables are not created.

## Theme tokens (RN-217)

| Tokens | Kind | Default |
| --- | --- | --- |
| `primary`, `on_primary`, `secondary`, `on_secondary`, `surface`, `on_surface`, `background`, `on_background`, `danger`, `on_danger`, `link` | Colour `#rrggbb` | An accessible neutral palette |
| `font_family` | Font families: letters, digits, spaces, commas, hyphens and quotes, at most 200 characters | `system-ui, sans-serif` |
| `font_size`, `spacing`, `radius` | Pixels within bounds (12–24, 2–16, 0–32) | 16, 4, 6 |
| `motion` | `full` or `reduced` | `full` |

Every `on_*` colour keeps a contrast of at least 4.5:1 on its background, and `link` on `surface` and `background` (WCAG 2.2 AA for normal text); the theme as it resolves in the scope that sets it is checked, and one that fails is refused. The client reads `GET /scopes/{scope}/theme.css`, the tokens as `--kampus-<token>` custom properties.

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Custom field values stored as text whatever their type | One typed column per type, with a check |
| Choices stored as newline-separated text in the field definition | A choice table with stable codes |
| Block placements addressed by page-type patterns and sub-page strings | A layout per page kind and scope, with typed placements |
| Rules written as code in plugins | Typed triggers, conditions and actions declared by modules |
