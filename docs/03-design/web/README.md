---
id: DESIGN-WEB
status: accepted
origin: requirements
sourceOfTruth: model
---

# Web client

The web client (`web/`) is a **Next.js** application in **React** and **TypeScript** (DEC-089). Its shell places the **UI modules** of every Kampus module and plugin into its **extension points** (DEC-004, DEC-074). It holds no business logic: everything comes from the API through a generated client (DEC-076). Its components come from `@kampus/ui`, a design system built on **MUI** and themed by Didactika's identity (DEC-013), which prevails over MUI's defaults. Requirement RF-008; RNF-003, RNF-010.

![UI modules and extension points](./ui-modules.svg)

## Packages

The client is one npm workspace at the root of the repository (`package.json`), so the shell, the shared packages and the UI module of every Kampus module build and test together:

| Package | Folder | Holds |
| --- | --- | --- |
| `@kampus/web` | `web/` | The Next.js application: the shell and the registry of UI modules |
| `@kampus/ui` | `web/ui/` | The design system: tokens, the MUI theme, components, patterns and icons |
| `@kampus/sdk` | `web/sdk/` | The generated API client and the UI module contract |
| `@kampus/<module>-ui` | `crates/<module>/web/` | The UI module of a Kampus module, next to its Rust crate, as a plugin ships both in one package (RN-204) |

`npm run check`, `npm test` and `npm run build` at the root run every package; CI runs them in the job *Web client*. Everything is TypeScript in strict mode; tests live in `tests/` of each package, never next to the code.

## Folder structure

Every package keeps one layout, so a module or a plugin looks the same as the shell from inside, and a new one starts from the same skeleton. Folders hold one concern each; a component is a folder named like it, with an `index.ts` that is its only entry.

```text
web/                                   @kampus/web, the Next.js application
  next.config.ts
  src/
    app/                               routing files of the App Router only: layout, page, loading, error, not-found
      layout.tsx                       the document: language, direction, theme of the scope, providers
      (auth)/sign-in/page.tsx          pages without the frame of the campus
      (campus)/layout.tsx              the frame: app bar, navigation, the region of toasts
      (campus)/page.tsx                the dashboard
      (campus)/[...path]/page.tsx      the pages of UI modules, found in the registry
    features/<feature>/                the screens and parts of the shell itself (session, navigation, preferences)
    modules/                           the generated registry of UI modules and the extension points
    theme/                             the theme of the scope as an MUI theme, and the scheme of the person
    i18n/                              the strings of the shell per language, the gathered texts and formatters
    api/                               the API client set up for the server and for the browser
  tests/
    unit/
    e2e/                               Playwright, one file per use case
  ui/                                  @kampus/ui, the design system
    tokens/                            reference, system, light and dark tokens (W3C)
    src/
      theme/                           tokens and the roles of a scope turned into the MUI theme
      components/<Component>/          Didactika's components over MUI: <Component>.tsx and index.ts
      patterns/<Pattern>/              Didactika's patterns: list toolbar, filters, table, pager, selection bar, wizard, empty state
      icons/                           Material Symbols by name, the registry and its overrides
      tokens/                          reading the tokens, contrast
    tests/
  sdk/                                 @kampus/sdk, the generated API client and the UI module contract
crates/<module>/web/                   @kampus/<module>-ui, the UI module of a Kampus module or plugin
  kampus-ui.json                       the manifest: routes, contributions, strings, events
  src/
    index.ts                           the module: its routes and contributions, typed by @kampus/sdk
    routes/<route>/                    one folder per page of the module
    contributions/<point>/             widgets, menu items, tabs, actions, form sections, settings panels
    components/<Component>/            components private to the module
    strings/<language>.json            the texts of the module, one file per language, keys under its name
  tests/
```

A module uses the components and patterns of `@kampus/ui`, or MUI directly with the theme; it never sets a colour, a font or a radius of its own, and never restyles a component of the design system.

## Shell

![Shell](./shell.svg)

| Part | Responsibility |
| --- | --- |
| Router | The App Router of Next.js: the pages of the shell, and the routes UI modules declare (`/courses/:id`, `/admin/settings`…) through the registry; server-side rendering of the first page, client navigation afterwards |
| Layout engine | Renders the page kind's regions from the resolved layout (RN-216); a widget whose module is disabled or failing is skipped with a placeholder |
| Navigation | Menus from data (DEC-071), filtered by permissions |
| Session | Signed-in account, current scope, language and time zone; reacts to revocation (DEC-031) |
| Theme | Turns the theme of the scope of the page into the MUI theme, in light and dark; the scheme of the person; direction from the language (RN-192) |
| Strings | Loads message bundles per module and language (RN-191), with terms applied |
| Help | Opens the help topic of the current screen and runs guided tours (DEC-077) |
| Error boundary | One per extension point: a failing UI module shows an error card and reports it, the rest of the page works |

## UI module contract

A UI module is a React package with a `kampus-ui.json` manifest, written with `@kampus/ui`. Each Kampus module and each plugin ships at most one.

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
| UI-01 | UI modules are built into the client from a registry generated from the installed modules (`tools/repo-check --fix`); each page and contribution is loaded lazily, only when it is shown. |
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

The look and behaviour of every screen, its principles (Didactika prevails, latest stable versions, scalable, dynamic, optimised, intuitive, accessible), its tokens, theme, components, patterns, icons and theming per scope are in the [Kampus design system](./design-system.md).

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
