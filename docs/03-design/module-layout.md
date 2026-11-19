---
id: DESIGN-MODULE-LAYOUT
status: accepted
origin: requirements
sourceOfTruth: model
---

# Module and crate layout

Every module crate, in this repository or outside it, has the same structure (DEC-087). The server reads only the module's descriptor; the layout tells developers, reviewers and `tools/repo-check` where each declaration lives.

## The descriptor

`src/module.rs` exposes the module as the server reads it:

```rust
pub const NAME: &str = "identity";

pub const MODULE: Module = Module {
    name: NAME,
    manifest: kampus_common::manifest!(),
    summary: "Accounts, credentials, sign-in, second factors, sessions and API tokens",
    steps: crate::db::STEPS,
    permissions: crate::permissions::PERMISSIONS,
    templates: crate::permissions::TEMPLATES,
    plugin_types: &[],
};
```

| Field | Comes from | The server |
| --- | --- | --- |
| `name` | `src/module.rs` | Names the module's database schema and owns its grants |
| `manifest` | `version.toml`, `Cargo.toml` and `assets/icon.svg`, read by `manifest!()` | Checks it and records the version (see [the version file](#the-version-file)) |
| `plugin_types` | `src/module.rs` | Accepts plugins named `<type>_<name>` in `crates/<module>/<type>/<name>` |
| `steps` | `src/db/steps.rs` | Applies the pending upgrade steps (DEC-082) |
| `permissions` | `src/permissions/mod.rs` | Synchronises them at start-up and retires those no module declares (RN-021) |
| `templates` | `src/permissions/templates.rs` | Merges the values of every module for each role template |

## The folders

| Path | Holds | Required |
| --- | --- | --- |
| `Cargo.toml` | The package `kampus-<module>`, with `[package.metadata.kampus] kind = "module"` | yes |
| `src/lib.rs` | The crate documentation, the service type and its constructor, the module list and re-exports | yes |
| `src/module.rs` | `NAME` and `MODULE` | yes |
| `src/db/` | `steps.rs`, the upgrade steps in the neutral schema (DEC-065); queries stay with the services that run them | yes |
| `src/permissions/` | `mod.rs`, the permission codes and declarations; `templates.rs`, the values given in role templates | yes |
| `src/model/` | Entities, identifiers and values | when it has any |
| `src/service/` | The service API, one file per area of use cases (`account.rs`, `sign_in.rs`) | when it has any |
| `src/ports/` | Traits that other modules implement for this one (`Courier`, `Policies`) | when it has any |
| `src/support/` | Internal helpers that are not part of the API (hashing, encodings) | when it has any |
| `src/jobs/` | One file per job | when it has any |
| `src/events/` | One file per event it publishes | from the events module (T-C.06) |
| `src/observers/` | One file per event of another module it handles | from the events module (T-C.06) |
| `src/settings/` | The settings it declares | from the settings module (T-C.06) |
| `src/http/` | Its routes and their request and response types | from the API module (T-C.12) |
| `src/privacy/` | Its export and erase handlers (DEC-061) | from the privacy module (T-C.12) |
| `tests/` | Integration tests against the real ports, one test per business rule or use case | yes |
| `web/` | Its part of the web client (DEC-004) | from the web client (T-C.13) |
| `lang/` | Its messages (DEC-062) | from the i18n module |
| `README.md` | Optional notes for developers | no |

Nothing else may sit at the top of the crate or of `src/`; `tools/repo-check` reports any other file or folder, a missing descriptor, and a registry out of date.

## Tests

Tests never live in the code: no `#[cfg(test)]` in `src/`. Each crate keeps them in `tests/`, in one folder per kind; each folder is one test binary whose `main.rs` declares one file per topic:

```text
tests/
  unit/          pure checks, without databases or services
    main.rs      mod keys; mod schema; …
    schema.rs
  integration/   against PostgreSQL and the services of Docker Compose
    main.rs
    rules.rs     one test per behaviour
  conformance/   adapters: the conformance suite of their port
    main.rs
  support/       helpers shared by the folders above, included with #[path]
```

Tests go through the public API of the crate, as any caller would. `tools/repo-check` rejects a `#[cfg(test)]` in `src/`, a file directly in `tests/`, any other folder, and a folder without its `main.rs`.

## Every other crate

Foundation, port, adapter and server crates follow the same principle: `src/` holds only its entry files, and everything else lives in a folder named after its concern, each with a `mod.rs`. Folder names are snake case. Public paths do not depend on folders: `lib.rs` re-exports what callers use.

| Kind | At the top of `src/` | Required folders | Also at the crate root |
| --- | --- | --- | --- |
| `foundation` | `lib.rs` | — | — |
| `port` | `lib.rs` | `port/` (the trait and its types), `registry/` (`Adapter` and `open`), `conformance/` (the suite every adapter passes) | `adapters/` |
| `adapter` | `lib.rs` with `ADAPTER` | `store/` (the implementation) | — |
| `server` | `lib.rs`, `main.rs` | `commands/`, `config/`, `http/`, `wiring/` (the generated registries and the composition) | `adapters.toml`, `modules.toml` |
| `module` | `lib.rs`, `module.rs` | `db/`, `permissions/`, and the folders above | `web/`, `lang/`, `assets/` |

The crates of the repository today:

| Crate | Folders of `src/` |
| --- | --- |
| `kampus-common` | `adapter/` (settings of adapters), `clock/` (instants and clocks), `error/` (the error model), `id/` (typed identifiers), `page/` (pagination), `request/` (request context) |
| `kampus-db` | `connection/` (pool, prefix, error classes), `schema/` (the neutral schema), `upgrade/` (steps and the runner), `lint/` (the schema linter) |
| `kampus-module` | `descriptor/` (`Module`), `permission/` (the vocabulary of declarations) |
| `kampus-storage`, `kampus-cache` | `port/`, `registry/`, `conformance/`; the cache adds `layered/` |
| `kampus-server` | `commands/` (serve, upgrade, check), `config/`, `http/`, `wiring/` |

`tools/repo-check` rejects a file at the top of `src/` that is not an entry file, a folder without `mod.rs`, a missing required folder, and a crate without its kind.

## Names and folders

The name of a component comes from its package, and its folder follows from its name:

| Component | Package | Name | Folder |
| --- | --- | --- | --- |
| Core module | `kampus-identity` | `identity` | `crates/identity` |
| Adapter of the `cache` port | `kampus-cache-redis` | `cache_redis` | `crates/cache/adapters/redis` |
| Plugin of a type that a module defines | `kampus-<type>-<name>` | `<type>_<name>` | `crates/<owner>/<type>/<name>` |

Plugins and adapters carry their type as prefix, so their names never collide with the core. Only plugins and adapters may come from outside the repository; they are declared in `crates/server/modules.toml` or `crates/server/adapters.toml`. A module that defines a plugin type lists it in `plugin_types` of its descriptor.

`tools/repo-check` checks that every package matches its folder. The server checks names, versions, supported releases and requirements before it installs, upgrades or starts, and refuses with the list of problems.

## The version file

Every component has a `version.toml` next to its `Cargo.toml`, in the same form as the platform's `version.toml` at the root. The descriptor reads it with `kampus_common::manifest!()`, so no version data is written in code:

```toml
[component]
version = "1.2.0"
maturity = "beta"
kampus = ">=0.1.0, <0.3.0"
date = "2026-11-19"

[requires]
access = ">=0.1.0"
```

```rust
manifest: kampus_common::manifest!(),
```

| Key | Default | Meaning |
| --- | --- | --- |
| `component.version` | required | The version, semantic; equal to the package version |
| `component.maturity` | `stable` | `alpha`, `beta`, `rc` or `stable` |
| `component.kampus` | any release | The Kampus releases it works with, as a Cargo version requirement |
| `component.date` | none | The date of its release, `YYYY-MM-DD` |
| `component.icon` | `assets/icon.svg` when it exists, otherwise a generic icon | Another path, relative to the crate, only when the icon lives elsewhere |
| `[requires]` | none | Other components it needs, each with the versions it works with |

Name, summary and licence come from `Cargo.toml`. Everything but the version is optional: a component without it builds and runs, and `kampus-server components` shows what it lacks. A version that differs from the package, an unknown maturity, a malformed date or an icon that does not exist stops the build with the file and the key. Core components share the Kampus version (`version.workspace = true`); external ones have their own.

## Versions and the registry

The installation records the installed version of every component and every run that changed it:

| Table | Holds |
| --- | --- |
| `component` | Each installed component: kind, origin, version, maturity, when it was installed and last changed |
| `upgrade_run` | Each upgrade, downgrade or uninstall: the releases before and after, when, and how it ended |
| `component_change` | What each run did to each component: versions before and after, and how many steps |

A component with a new version needs an upgrade, even when it has no new steps. A component installed in a newer version than the build, or installed but missing from the build, stops the start: the operator downgrades, adds it back, or uninstalls it.

## Install actions and data migrations

A step may carry an action: Rust code that runs after the step's changes, in the same transaction, once. The action of step 1 is what the component sets up when it is installed; actions of later steps migrate data:

```rust
Step {
    sequence: 1,
    name: "create notes",
    revert: &[],
    action: Some(Action { name: "first note", run: first_note }),
    changes: &[/* tables */],
}

fn first_note<'c>(conn: &'c mut PgConnection, context: &'c ActionContext) -> ActionFuture<'c> {
    Box::pin(async move {
        sqlx::query(sqlx::AssertSqlSafe(format!("insert into {} (id) values ($1)", context.table("note"))))
            .bind(uuid::Uuid::now_v7())
            .execute(conn)
            .await
            .map(drop)
            .map_err(kampus_db::classify)
    })
}
```

If the action fails, the step's changes are undone with it and the run is recorded as failed. The action's name is part of the step's checksum, so renaming it after a release counts as changing the step.

## Going back

Versions never go back by themselves. A step that can be undone says how in `revert`; the SQL is kept in the installation when the step is applied, so an older build can undo steps it has never seen:

```rust
Step {
    sequence: 2,
    name: "add note text",
    revert: &[Change::DropColumn { table: "note", column: "body", unused_since: "1.1.0" }],
    action: None,
    changes: &[Change::AddColumn { table: "note", column: Column::new("body", Type::Text).nullable() }],
}
```

`kampus-server downgrade --yes`, run with the older build, undoes every newer step, newest first, and takes the components back to the build's versions. When a newer step left no way back, nothing is undone and the command names the steps. Actions are not undone: a step whose action changes data must say in `revert` how to bring the data back, or leave `revert` empty.

## Uninstalling

`kampus-server uninstall <component> --yes` removes a component that is installed but no longer in the build: its tables, its steps in the ledger and its registration. It is refused while a component of the build requires it.

## Commands of the operator

| Command | Does |
| --- | --- |
| `kampus-server components [--json]` | Lists the components of the build and of the installation: versions, maturity, state and what each lacks |
| `kampus-server history [--limit N]` | Shows the latest upgrades, downgrades and uninstalls, with what each did to each component |
| `kampus-server check` | Says whether the installation matches the build, without changing it |
| `kampus-server upgrade` | Applies the pending steps and registers the components |
| `kampus-server downgrade --yes` | Goes back to the build's versions |
| `kampus-server uninstall <component> --yes` | Removes a component no longer in the build |

## Registering a module

The server's registry, `crates/server/src/wiring/registry.rs`, lists every module of the build in dependency order, and the `# modules:start` block of `crates/server/Cargo.toml` depends on each. `tools/repo-check --fix` writes both:

- from every crate of kind `module` under `crates/`, ordered by the module table of the [architecture](./architecture.md#modules);
- then from `crates/server/modules.toml`, for modules published outside the repository:

```toml
[[module]]
crate = "kampus-attendance"
source = 'git = "https://example.org/kampus-attendance", tag = "v1.0.0"'
```

An external module depends only on published Kampus crates, follows this layout, and passes the same checks.

## Adding a kind of contribution

A new kind (events, settings, routes…) is introduced by the core module that owns it, in one pull request:

1. a field in `kampus_module::Module`, documented with the folder it comes from;
2. the folder in `SRC_FOLDERS` of `tools/repo-check/modules.mjs`, and, when every module must have it, a line in `REQUIRED`;
3. what the server does with it, in `crates/server/src/modules.rs`;
4. its row in the tables above, and its section in the platform and plugin developer guides.
