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
| `page_kind` | `code` PK, `module`, `summary`, `grid` jsonb (`columns`, `row`: `compact`, `regular` or `tall`), `retired_at`; `regions`, no longer written, is dropped once no supported release reads it | Synchronised from declarations (DEC-090) |
| `widget` | `code` PK, `module`, `summary`, `sizes` jsonb (for each page kind it goes on: `min`, `default` and `max` in cells and `resize`), `settings` jsonb (key, stored kind and default of each), `retired_at`; `page_kinds`, no longer written, is dropped once no supported release reads it | Synchronised from declarations; a size is kept only for a page kind of the build whose grid holds it, and a widget goes on no page kind it has no size for; a retired widget is skipped when a layout is read (RN-216) |
| `menu_kind` | `code` PK, `module`, `summary`, `retired_at` | Synchronised from declarations |
| `layout` | `id`, `page_kind_code` FK → `page_kind` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `locked`, `version` | `UNIQUE` (`page_kind_code`, `scope_id`); the layout of the nearest scope applies whole, unless a scope above locks its own |
| `placement` | `id`, `layout_id` FK (`CASCADE`), `widget_code` FK → `widget` (`CASCADE`), `x`, `y` (`CHECK` ≥ 0), `w`, `h` (`CHECK` > 0), `settings` jsonb (validated against the widget's settings), `permission` (who sees it; everyone when null); `region` and `position`, no longer written, are dropped once no supported release reads them | Replaced with the whole layout; a rectangle stays within the grid and the sizes of its widget, and no two overlap (DEC-090). Read in reading order; a widget whose sizes changed since takes the nearest size it allows |
| `menu` | `id`, `menu_kind_code` FK → `menu_kind` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `locked`, `version` | `UNIQUE` (`menu_kind_code`, `scope_id`) |
| `menu_item` | `id`, `menu_id` FK (`CASCADE`), `parent_id` FK → `menu_item` (`CASCADE`), `label`, `target` (a path of the site or an `http`/`https` address), `permission`, `position` | Index (`menu_id`, `parent_id`, `position`); at most 100 items, 3 levels deep; a hidden item hides its children. Labels per language arrive with languages |
| `theme` | `id`, `scope_id` FK → `access.scope` (`CASCADE`), `tokens` jsonb (the tokens this scope sets, validated, RN-217), `locked`, `version` | `UNIQUE` (`scope_id`); tokens merge from the platform down to the scope read, stopping at the first scope that locks; the parent of a theme is the theme of the scope above, so it needs no column |
| `rule` | `id`, `scope_id` FK → `access.scope` (`CASCADE`), `owner` (the principal it acts as, who wrote it last; not a foreign key), `name`, `trigger` (a declared event type), `conditions` jsonb (typed tree of `all`, `any`, `not` and `compare` on the fields the trigger declares), `actions` jsonb (declared actions with typed parameters), `enabled`, `version`, `created_at` | Index (`trigger`, `enabled`), index on `scope_id`; reacts to the events of its scope and the scopes below (RN-218) |
| `rule_run` | `id`, `rule_id` FK (`CASCADE`), `event` (soft reference: events are purged), `outcome` (`running`, `succeeded`, `failed`, `skipped`), `actions_done`, `error`, `at` | `UNIQUE` (`rule_id`, `event`): one evaluation per event, which a redelivery resumes after its last done action; index (`rule_id`, `at`) (RN-219) |
| `flag` | `code` PK, `module`, `summary`, `default_value` boolean, `retired_at` | Synchronised from declarations |
| `flag_value` | `flag_code` FK (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `enabled`, `locked` | PK (`flag_code`, `scope_id`) |
| `change` | `id`, `kind` (`term`, `flag`, `field`, `layout`, `menu`, `theme`, `rule`, `package`), `object` (for a package, its checksum), `scope_id` FK → `access.scope` (`CASCADE`), `actor` (not a foreign key, it outlives the account), `before` jsonb, `after` jsonb, `at` | Append-only (RN-222); index (`scope_id`, `at`); every change also publishes `customise.changed`; an applied package is one change |
| `package_import` | `id`, `scope_id` FK → `access.scope` (`CASCADE`), `actor`, `checksum` (SHA-256 of the package as written), `package` jsonb, `differences` jsonb, `created_at`, `applied_at` | Index (`scope_id`, `created_at`); applied at most once (RN-221) |

## Packages (RN-221)

A package holds what a scope **itself** sets of the kinds chosen (terms, flags, fields without their values, layouts, menus, theme, rules), never what it takes from above, in format `1`, with the SHA-256 of its contents. Reading it into a scope validates every item against the declarations, locks and permissions there (`customise.package.manage` and the permission of each kind it holds) and keeps it with its **differences**: what it adds and what it changes, with the value before. Applying it checks it again, since the scope may have changed, and writes everything in one transaction recorded as one change. An import adds and changes; it never removes. Fields are matched by kind and code, and a field whose type differs is refused; rules are matched by name, and the importer becomes their owner.

Rules listen to `events.any`, the type `kampus-events` declares so that a listener receives every event. An action runs after the event committed, as the owner of the rule, who must hold the permission the action declares in the scope of the event; a failure is recorded and publishes `customise.rule_failed`, and is not retried. A rule cannot be triggered by an event its actions declare they publish, and no rule runs for an event more than five steps down a chain.

## Theme tokens (RN-217)

| Tokens | Kind | Default (Didactika, DEC-013) |
| --- | --- | --- |
| `brand`, `primary`, `secondary`, `accent`, `neutral`, `danger`, `success`, `warning`, `info` | Seed colour `#rrggbb` | `#00555a`, `#0f766e`, `#475569`, `#06b6d4`, `#64748b`, `#b91c1c`, `#15803d`, `#b45309`, `#0369a1` |
| `font_family`, `font_family_brand` | Font families: letters, digits, spaces, commas, hyphens and quotes, at most 200 characters | `"Inter Variable", Inter, system-ui, -apple-system, "Segoe UI", sans-serif` for both |
| `font_size` | Pixels, 12–24 | 16 |
| `radius` | Pixels, 0–28: the medium corner; the other corners are multiples of it | 12 |
| `density` | `comfortable` or `compact` | `comfortable` |
| `motion` | `full` or `reduced` | `full` |
| `scheme` | `system`, `light` or `dark`: the scheme the scope prefers | `system` |

Each seed colour grows a **tonal palette** of eleven tones (50 to 950). When the seed is a tone of one of Didactika's palettes, that palette is used; otherwise the palette is grown in OKLCH with the hue of the seed, and the seed takes the place of the tone nearest to it. The theme gives the **colour roles** of a light and a dark scheme, after Material 3:

| Roles | Light | Dark |
| --- | --- | --- |
| `<colour>` (`primary`, `secondary`, `accent`, `danger`, `success`, `warning`, `info`) | The seed when it keeps 4.5:1 on `surface`, otherwise the first darker tone that does | Tone 300, or the first lighter tone that keeps 4.5:1 |
| `on-<colour>` | White or tone 950, whichever contrasts more | The same |
| `<colour>-container`, `on-<colour>-container` | Tones 100 and 900 | Tones 800 and 100 |
| `brand`, `on-brand` | The seed, and white or the darkest neutral | The same |
| `surface`, `surface-container-lowest` to `-highest` | Neutral 50; white, 100, 100–200, 200, 200–300 | Neutral 900; 950, 900–800, 800, 800–700, 700 |
| `on-surface`, `on-surface-variant`, `outline`, `outline-variant` | Neutral 900, 600, 500, 300 | Neutral 50, 300, 400–300, 700 |
| `inverse-surface`, `inverse-on-surface`, `inverse-primary`, `scrim`, `shadow` | Neutral 800, 100, the dark `primary`, neutral 950, primary 950 | Neutral 100, 800, the light `primary`, neutral 950, neutral 950 |

Every `on-*` role keeps 4.5:1 on its background, each colour role 4.5:1 on `surface`, and `outline` 3:1 on every surface. The theme is checked as it resolves in the scope that sets it, and a theme that fails is refused (`customise.theme_contrast`); the way the roles are grown prevents that. Neutral palettes grown from a seed follow a darker lightness curve than colours, as surfaces and their text need.

The client reads `GET /scopes/{scope}/theme.css`, written in the custom properties of its design system:

- the roles of the preferred scheme on `:root`, as `--k-color-<role>`;
- the other scheme under `prefers-color-scheme` and `[data-theme]`;
- `--k-font-family-plain`, `--k-font-family-brand`, `--k-font-size-root`, `--k-radius`, `--k-density` and `--k-motion`.

So a theme replaces the whole look.

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| Custom field values stored as text whatever their type | One typed column per type, with a check |
| Choices stored as newline-separated text in the field definition | A choice table with stable codes |
| Block placements addressed by page-type patterns and sub-page strings | A layout per page kind and scope, with typed placements |
| Rules written as code in plugins | Typed triggers, conditions and actions declared by modules |
