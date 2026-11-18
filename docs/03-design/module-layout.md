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
    summary: "Accounts, credentials, sign-in, second factors, sessions and API tokens",
    steps: crate::db::STEPS,
    permissions: crate::permissions::PERMISSIONS,
    templates: crate::permissions::TEMPLATES,
};
```

| Field | Comes from | The server |
| --- | --- | --- |
| `name` | `src/module.rs` | Names the module's database schema and owns its grants |
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
