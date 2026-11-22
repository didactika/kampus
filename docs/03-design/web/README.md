---
id: DESIGN-WEB
status: accepted
origin: requirements
sourceOfTruth: model
---

# Web client

The web client (`web/`) is an **application shell** that loads the **UI modules** of every Kampus module and plugin at runtime into its **extension points** (DEC-004, DEC-074). It holds no business logic: everything comes from the API through a generated client (DEC-076). Its components are framework-agnostic Web Components themed by design tokens (DEC-075). Requirement RF-008; RNF-003, RNF-010.

![UI modules and extension points](./ui-modules.svg)

## Packages

The client is one npm workspace at the root of the repository (`package.json`), so the shell, the shared packages and the UI module of every Kampus module build and test together:

| Package | Folder | Holds |
| --- | --- | --- |
| `@kampus/web` | `web/` | The shell |
| `@kampus/ui` | `web/ui/` | The design system: tokens and Web Components |
| `@kampus/sdk` | `web/sdk/` | The generated API client and the UI module contract |
| `@kampus/<module>-ui` | `crates/<module>/web/` | The UI module of a Kampus module, next to its Rust crate, as a plugin ships both in one package (RN-204) |

`npm run check`, `npm test` and `npm run build` at the root run every package; CI runs them in the job *Web client*.

## Folder structure

Every package keeps one layout, so a module or a plugin looks the same as the shell from inside, and a new one starts from the same skeleton.

```text
web/                          @kampus/web, the shell
  src/
    app/                      start-up, router, layout engine, error boundary
    modules/                  loading UI modules through the import map, extension points
    session/                  account, scope, language, time zone
    theme/                    the theme of the current scope and the scheme of the person
    strings/                  message bundles and formatters
    pages/                    the few pages of the shell itself (sign in, not found, errors)
  tests/
  ui/                         @kampus/ui, the design system
    tokens/                   reference.tokens.json, system.tokens.json, light.tokens.json, dark.tokens.json
    src/
      components/<name>/      one folder per element: <name>.ts, and <name>.styles.ts when its styles are long
      forms/                  the base of the controls that take part in forms
      styles/                 what every element shares: base, field, state layer, tones, type scale
      tokens/                 tokens as custom properties, contrast
    tests/
  sdk/                        @kampus/sdk, the generated API client and the UI module contract
crates/<module>/web/          @kampus/<module>-ui, the UI module of a Kampus module or plugin
  kampus-ui.json              the manifest: routes, contributions, strings, events
  src/
    index.ts                  registers what the manifest declares
    routes/<route>/           one folder per page of the module
    contributions/<point>/    widgets, menu items, tabs, actions, form sections, settings panels
    components/<name>/        elements private to the module, named k-<module>-<name>
    strings/                  the message keys of the module
  tests/
```

A module uses the elements of `@kampus/ui` and its tokens; it defines an element of its own only for what is particular to it, with the same tokens, and never restyles one of the design system.

## Shell

![Shell](./shell.svg)

| Part | Responsibility |
| --- | --- |
| Router | Routes declared by UI modules (`/courses/:id`, `/admin/settings`…); server-side rendering of the first page, client navigation afterwards |
| Layout engine | Renders the page kind's regions from the resolved layout (RN-216); a widget whose module is disabled or failing is skipped with a placeholder |
| Navigation | Menus from data (DEC-071), filtered by permissions |
| Session | Signed-in account, current scope, language and time zone; reacts to revocation (DEC-031) |
| Theme | Applies the resolved design tokens as CSS custom properties (RN-217); direction from the language (RN-192) |
| Strings | Loads message bundles per module and language (RN-191), with terms applied |
| Help | Opens the help topic of the current screen and runs guided tours (DEC-077) |
| Error boundary | One per extension point: a failing UI module shows an error card and reports it, the rest of the page works |

## UI module contract

A UI module is an ES module package with a `kampus-ui.json` manifest, built with any framework but using the design system. Each Kampus module and each plugin ships at most one.

| Manifest section | Content |
| --- | --- |
| `module` | id (the module or plugin id), version, SDK range, entry file |
| `routes` | path, page kind, permission required to show it, help topic |
| `contributions` | widgets (for regions), menu items, page slots (tabs, panels, actions), form sections for other modules' forms, settings panels |
| `strings` | the message keys the module uses (checked against its declared messages) |
| `events` | client events it emits and listens to |

Rules:

| ID | Rule |
| --- | --- |
| UI-01 | The shell loads a UI module through the import map only when a route or contribution of it is needed; nothing is bundled into the shell. |
| UI-02 | A contribution is shown only when its module is enabled in the scope (RN-209, RN-220) and the viewer holds its permission; the API remains the authority (DEC-017). |
| UI-03 | Extension points have typed props and events; a module never reaches into another module's DOM or state. |
| UI-04 | Every route and contribution declares a help topic (DEC-077); the build fails without it. |
| UI-05 | Every text is a message key (RN-187); every date, number and money value goes through the locale formatter (RN-192). |
| UI-06 | UI modules talk to the API only through the generated client; they never build URLs by hand. |

## Extension points

| Point | Where | Filled with |
| --- | --- | --- |
| `region:<page kind>:<region>` | regions of every page kind (for example `course:main`, `course:side`, `dashboard:top`) | widgets |
| `menu:<menu kind>` | primary navigation, course navigation, user menu, admin menu | menu items |
| `slot:<page kind>:tabs` | tabs of an entity page (course, activity, profile) | panels |
| `slot:<page kind>:actions` | action menus of entities | actions |
| `form:<form id>:sections` | forms of other modules (course settings, profile, enrolment method) | form sections with their own validation |
| `settings:<scope kind>` | settings pages | settings panels |
| `activity:<type>` | the view and editor of a course module | the activity type's pages (RN-075) |

## Design system

The design system follows **Material 3** (colour roles, tonal palettes, type scale, shape, elevation, state layers, motion and density) with **Didactika's identity** as its default theme (DEC-013), and Didactika's rules of meaning for each element.

| Aspect | Rule |
| --- | --- |
| Components | Web Components (`k-button`, `k-input`, `k-textarea`, `k-select`, `k-checkbox`, `k-card`, `k-alert`, `k-badge`, `k-spinner`, `k-dialog`, `k-toasts`, `k-tabs`, and later `k-table`, `k-menu`, `k-date-picker`, `k-rich-text`, `k-file-picker`…), published as `@kampus/ui`; controls take part in forms through `ElementInternals` |
| Tokens | W3C design tokens in four documents: the **reference** palettes (Didactika's teal, slate, cyan and the colours of state), the **system** tokens (typefaces, type scale, corners, density, sizes, spacing, elevation, state layers, easing, durations, gradients, layers), and the **colour roles** of the light and the dark scheme. Only system tokens and roles become custom properties (`--k-…`); elements use roles, never a palette or a literal colour |
| Colour | Material 3 roles: `primary`, `secondary`, `accent` (the tertiary role), `danger`, `success`, `warning`, `info`, each with `on-`, `-container` and `on-…-container`; `brand`; surfaces from `surface-container-lowest` to `-highest`; `on-surface`, `on-surface-variant`, `outline`, `outline-variant`; inverse roles, `scrim` and `shadow`. Light and dark schemes; the page follows the system unless the theme or the person chooses (`data-theme`) |
| Type | The Material 3 scale (display, headline, title, body, label; large, medium, small) in rem, so it follows the size of the theme. Display, headlines and large titles in Space Grotesk; everything else in IBM Plex Sans |
| Shape | Corners from one radius: extra small, small, medium (the radius), large, extra large and full are multiples of it, so a radius of 0 makes every shape square and buttons stop being pills |
| Elevation and states | Five levels of shadow tinted with the theme; a state layer of the colour of the content shows hover, focus and press; disabled elements use the Material opacities |
| Density and motion | Comfortable or compact (every control two steps of 4 px smaller); durations scale with the motion of the theme and stop under reduced motion |
| Meaning | One meaning per element: one `filled` button per area for its main action, `outlined` for ordinary actions, `text` for the least; `danger` removes or stops and always asks first; `success` starts a process; badges are tinted with their state; the name of a row opens it; an action a row cannot take is disabled, never hidden |
| Patterns | Lists with a toolbar (a search that applies as the person types, a Filters button with the number of filters on, the actions of the page at its end), a table of fixed height with a sticky head and a pager under it (12, 24, 48, 96 and All up to 500 rows), a floating bar for a selection, wizards with a stepper, dialogs that say what stays, empty states that say why and what to do |
| Accessibility | WCAG 2.2 AA: keyboard operation, visible focus, roles and names, contrast of every role checked in CI in both schemes, reduced motion honoured, forms with labelled errors, live regions for asynchronous results |
| Internationalisation | Logical CSS properties (`margin-inline-start`) so right-to-left works without overrides; every text comes from the caller, never from an element |
| Breakpoints | The window classes of Material 3: compact (under 600 px), medium (600–839), expanded (840–1199), large (1200–1599) and extra large; layouts from 320 px wide |

## Theming per scope

The look is configuration in every scope (RN-217): the platform, a tenant, a campus, a category or a course sets the tokens of its theme, and a scope below inherits them unless it sets its own or a scope above locks them.

| Step | What happens |
| --- | --- |
| Default | `@kampus/ui/tokens.css` holds Didactika's theme in both schemes |
| A scope sets tokens | Seed colours, typefaces, size, radius, density, motion and preferred scheme; the server grows the roles of both schemes, keeping WCAG 2.2 AA contrast |
| The shell shows a page | It adds `GET /api/v1/customise/scopes/{scope}/theme.css` of the scope of the page after the default; its custom properties are the same, so they replace the default without code |
| The person chooses | Light, dark or the system, stored with the person and applied as `data-theme` on the root |

The default theme and the theme the server grows for the default tokens are the same; a test of the customisation module compares them.

## Data and state

| Concern | Rule |
| --- | --- |
| API client | Generated from the OpenAPI document (RN-195); typed requests and problem details (DEC-018) |
| Caching | Per resource, revalidated with ETag; changes send `If-Match` and show conflicts to the person (DEC-026) |
| Repeated actions | Mutations carry an `Idempotency-Key`, so a double click never creates twice |
| Live updates | WebSocket subscription to the events the viewer may see, for notifications, messages and activity states |
| Offline | Out of scope for the first release (RF-117) |

## Quality

Playwright end-to-end tests per use case, automated accessibility scans on every page (axe) plus a manual audit per release (RNF-003), visual regression of the design system, and bundle budgets per UI module. Wireframes of each use case are in its requirements card.
