---
id: DEC-087
status: accepted
---

# DEC-087 One layout for every module crate, and a descriptor the server reads

- Date: 2026-11-18
- Context: DEC-015 asks every module to declare what it contributes through one contract, but did not say where each declaration lives in a crate. With two modules built, files sat side by side in `src/` (schema, declarations, services, helpers), and each new module, in the repository or outside it, would choose its own arrangement. The owner wants one structure for every crate from the start: everything a module manages for the database in one place, events, observers and services in theirs, the front apart, a structure the core reads to install and override, and a documented way to extend it.
- Options considered: a convention left to reviews; discovering contributions from file names (refused by DEC-015); a typed descriptor per module plus a fixed folder layout checked on every build (chosen).
- Decision:
  - A new foundation crate, `kampus-module`, holds the contract: the `Module` descriptor and the vocabulary of declarations (scope kinds, permission values, risks, permissions and role templates), which access re-exports.
  - Every module crate exposes `pub const MODULE: Module` in `src/module.rs`. The server installs and synchronises a module only from this descriptor: its upgrade steps, its permissions and its role template values today; events, observers, jobs, settings, routes, privacy handlers, web modules and help topics as their core modules arrive. Nothing is discovered from file names.
  - Every module crate follows one layout, so each declaration has one place: `src/lib.rs` and `src/module.rs` are the only files at the top of `src/`; `db/` holds the upgrade steps (`db/steps.rs`); `permissions/` the permissions (`mod.rs`) and template values (`templates.rs`); `model/` the entities and values; `service/` the service API, one file per area of use cases; `ports/` the traits other modules implement; `support/` internal helpers; `jobs/`, `events/`, `observers/`, `settings/`, `http/` and `privacy/` their contributions. Outside `src/`: `tests/`, `web/` (its part of the web client), `lang/` (its messages) and an optional `README.md`.
  - The other kinds of crate follow the same principle (owner, 2026-11-18): only entry files at the top of `src/` (`lib.rs`, plus `main.rs` for the server), every concern in its own folder with a `mod.rs`, and required folders per kind (ports: `port/`, `registry/`, `conformance/`; adapters: `store/`; server: `commands/`, `config/`, `http/`, `wiring/`).
  - Every crate states its kind in `[package.metadata.kampus] kind`: `foundation`, `port`, `adapter`, `module` or `server`.
  - The server's module registry (`crates/server/src/wiring/registry.rs` and its manifest's `modules` block) is written by `tools/repo-check --fix` from the module crates, in the order of the module table of the architecture, followed by the modules declared in `crates/server/modules.toml`, which come from outside the repository. CI fails when the registry is stale or a module breaks the layout.
  - A new kind of contribution is introduced by the core module that owns it: one field in `Module`, one folder in the layout and in `tools/repo-check`, and its section in the module layout guide, in the same pull request; from then on every module, inside the repository or not, uses it the same way.
- Consequences: Reading a module starts at `module.rs`; a reviewer knows where any declaration must be; a module written outside the repository plugs in with one declaration; the server never grows a hand-written list of modules.
- Affected artifacts: `crates/module`, every module crate, `crates/server`, `tools/repo-check`, the architecture, the [module layout](../../03-design/module-layout.md), the platform and plugin developer guides.
