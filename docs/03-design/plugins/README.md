---
id: DESIGN-PLUGINS
status: accepted
---

# Plugins: types, folders and rules

A **plugin** extends Kampus at an extension point that a **plugin type** defines: a widget for pages, an activity for courses, an outline for the content of a course. A plugin of the core and an external one have the same structure, follow the same rules and are installed, upgraded and removed the same way (DEC-087, DEC-091, DEC-092). This guide is the reference for where each plugin goes and what it may depend on; there is one page per plugin type.

## The folder rule

Every plugin type has a crate of its own, and its plugins live inside it:

```text
crates/<type>/                 the plugin type: what its plugins declare and the rules they keep (kind `type`)
├── src/declaration/           the vocabulary of a plugin's declaration
├── src/conformance/           the rules every plugin passes, run by its own tests and by the server's
├── web/                       the contract of the web part of its plugins, when it has one
└── plugins/<name>/            one plugin: package `kampus-<type>-<name>`, component `<type>_<name>`
    ├── Cargo.toml             kind `module`; depends on core modules and on `kampus-<type>` only
    ├── version.toml           its version, maturity, the Kampus releases it works with and what it requires
    ├── src/                   the module layout: `lib.rs`, `module.rs`, `db/`, `permissions/`, `settings/`…
    ├── tests/unit/            at least `conformance.rs`, which runs the rules of its type on its descriptor
    ├── web/                   its web part, a package of the workspace: `@kampus/<type>-<name>`
    │   ├── package.json
    │   ├── src/index.ts       what it gives the web client
    │   └── strings/<lang>.json its texts, every key under `<type>_<name>.`
    └── assets/icon.svg        its icon, optional
```

The folder `plugins/` has the same name in every type. Adapters of the ports are the one exception, kept as they are: a port chooses one adapter by configuration (`crates/<port>/adapters/<name>`, DEC-086), while a type has many plugins working at once.

## Plugin types

| Type | Crate | Extension point | Plugins in the repository | State |
| --- | --- | --- | --- | --- |
| [Widget](./widget.md) | `kampus-widget` | Pages arranged as grids: the home, the page of a course (DEC-090) | welcome, text, catalogue, notifications, course content, course summary, course progress, course grades, upcoming | built |
| Outline | `kampus-outline` (to come) | How the content of a course presents its sections and activities; the content widget delegates to the outline the course chooses (DEC-092) | sections (standard) | designed |
| Activity | `kampus-activity` (to come) | What a course holds: assignments, quizzes, pages, files… rendered inside the outline (RN-075, DEC-092) | — | designed; the contract lives in `activities` until its first plugin |
| Enrolment method | `kampus-method` (to come) | Ways into a course (DEC-040) | — | contract in `enrol` |
| Authentication | `kampus-auth` (to come) | Steps of the sign-in pipeline (DEC-030) | — | planned |
| Report | `kampus-report` (to come) | Reports over the data of the campus | — | planned |
| Question | `kampus-question` (to come) | Types of question of the question bank | — | planned |
| Theme | `kampus-theme` (to come) | Design tokens and assets, no code (RN-217) | — | planned |
| Local | `kampus-local` (to come) | Listeners, jobs, routes and rule actions anywhere | — | planned |

A type whose contract still lives in a module (activity in `activities`, method in `enrol`) moves to its own crate when its first plugin arrives, in the same pull request.

## Dependencies: clear and one way

- A plugin depends on **core modules** and on **the crate of its type**, never on another plugin. `tools/repo-check` refuses anything else.
- Only `kampus-server` depends on plugins: its registry is written from the plugin folders by `tools/repo-check --fix`, after the core modules.
- When a plugin needs what another type offers, it asks the **core**, which owns the registry of that type: the content widget asks `courses` for the outline of the course, and the outline asks `activities` for how each activity shows. The core reuses its services, permissions, cache and events, so a plugin never reimplements them.
- The web part of a plugin imports only `@kampus/ui`, `@kampus/sdk` and the host API of the client: `@kampus/web/host` on the server and `@kampus/web/host/client` in the browser (a client component may still import types from the server host). The host exports only what plugins may rely on; the rest of the client may change at any time.

## Installing, removing and external plugins

- A plugin of the repository is installed by its folder: the registries of the server and of the web client are written from the folders, so adding or removing a plugin is adding or removing its folder.
- A plugin from outside the repository has the same structure and is declared in `crates/server/modules.toml` with its git or registry source; its web part is a package of the same name. Installing one from a git repository, and going back to a version the code registers, will be offered by the administration with the settings module.
- Removing a plugin leaves the data that pointed to it unused, never broken: a layout skips a widget no plugin declares (RN-216).

## Checklist for any plugin

1. The folder and the package follow the rule above, and `version.toml` says what it requires.
2. `tests/unit/conformance.rs` runs the rules of its type and passes.
3. Its texts are in `web/strings`, in English and in every language of the repository, under its prefix.
4. Its web part imports only the host API, and `tools/repo-check` is green.
5. Its page in this guide or the guide of its type says what it does, and the CHANGELOG says it arrived.
