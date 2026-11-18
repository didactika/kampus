---
id: DESIGN-WEB
status: accepted
origin: requirements
sourceOfTruth: model
---

# Web client

The web client (`web/`) is an **application shell** that loads the **UI modules** of every Kampus module and plugin at runtime into its **extension points** (DEC-004, DEC-074). It holds no business logic: everything comes from the API through a generated client (DEC-076). Its components are framework-agnostic Web Components themed by design tokens (DEC-075). Requirement RF-008; RNF-003, RNF-010.

![UI modules and extension points](./ui-modules.svg)

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

| Aspect | Rule |
| --- | --- |
| Components | Web Components (`k-button`, `k-field`, `k-table`, `k-dialog`, `k-tabs`, `k-menu`, `k-toast`, `k-date-picker`, `k-rich-text`, `k-file-picker`…), published as `@kampus/ui` |
| Tokens | W3C design tokens: colour, typography, spacing, radius, elevation, motion; the default is Didactika's identity (DEC-013); themes override them (RN-217) |
| Accessibility | WCAG 2.2 AA: keyboard operation, visible focus, roles and names, contrast checked in CI, reduced motion honoured, forms with labelled errors, live regions for asynchronous results |
| Internationalisation | Logical CSS properties (`margin-inline-start`) so right-to-left works without overrides |
| Density and breakpoints | Comfortable and compact densities; layouts from 320 px wide |

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
