---
id: DEC-088
status: accepted
---

# DEC-088 Components with versions, a registry and an explicit way back

- Date: 2026-11-19
- Context: The owner wants every module, plugin and adapter, core or external, to carry its version and what an administrator needs to see (maturity, supported releases, requirements, icon), the installation to record the installed version of each and every upgrade in order, actions to run when a component is installed or upgraded, versions never to go back silently, and component names and folders bound to each other, all better managed than the version files of older platforms.
- Options considered: a version file per component read at run time; versions only in the ledger of steps; a manifest built from `Cargo.toml` at compile time, a registry of components with a history of runs, and data actions and undo SQL inside the steps (chosen).
- Decision:
  - Every component has a manifest (`kampus_common::component::Manifest`) written with `kampus_common::manifest! { … }`: the package name, version, description and licence come from its `Cargo.toml`; the icon from `assets/icon.svg` when the file exists; maturity (stable when not said), the Kampus releases it supports and the components it requires (with version requirements) are optional. Nothing optional blocks a build or a start; `kampus-server components` shows what is missing.
  - The name of a component is its package without `kampus-`: `kampus-identity` is `identity`. Plugins carry their type as prefix: `kampus-<type>-<name>` is `<type>_<name>`, and lives in the folder of the component that defines the type, `crates/<owner>/<type>/<name>`; adapters are the plugins of a port, in `crates/<port>/adapters/<name>`. Only plugins and adapters come from outside the repository. `tools/repo-check` checks packages against folders; the server checks names, versions, supported releases and requirements before it installs, upgrades or starts, and refuses with every problem listed.
  - The platform has its version file, `version.toml` at the root (name, version, maturity, date, `upgrade_from`), read by the core when it is built (`kampus_common::PLATFORM`). The platform is a component of its own, `kampus`, which owns the steps of the database layer; an installation newer than the build, or older than `upgrade_from`, is refused with what to do.
  - The installation keeps a registry: `component` (name, kind, origin, version, maturity, dates), `upgrade_run` (operation, releases, times, outcome, error) and `component_change` (what each run did to each component, with the steps it applied or undid). Every upgrade, downgrade and uninstall is recorded, failed ones too.
  - A new component version is an upgrade even without new steps. A component installed in a newer version than the build, or installed but missing from the build, stops the start.
  - A step may carry an action, Rust code run after its changes in the same transaction: the action of step 1 sets the component up when it is installed, later ones migrate data. The action's name is part of the step's checksum.
  - A step may carry the changes that undo it. They are rendered and kept in the ledger when the step is applied, so an older release can undo steps it never knew. Going back is explicit, `kampus-server downgrade --yes`; it is refused, naming the steps, when any newer step left no way back.
  - `kampus-server uninstall <component> --yes` removes a component that is no longer in the build, unless a component of the build requires it.
- Consequences: An administrator sees every component with its versions and the history of the installation; an upgrade, a downgrade or an uninstall never happens by surprise; a component's folder, package and name always agree.
- Affected artifacts: `crates/macros`, `crates/common`, `crates/db`, `crates/module`, every component, `crates/server`, `tools/repo-check`, the [module layout](../../03-design/module-layout.md), the operator and developer guides.
