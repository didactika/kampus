---
id: DEC-086
status: accepted
---

# DEC-086 Adapters as sub-plugins of their port, registered by scheme; folders without the project prefix

- Date: 2026-11-18
- Context: Adapters lived as sibling crates (`kampus-cache-moka`, `kampus-cache-redis`) next to every module, and every folder repeated the project name. The owner wants adapters to be sub-plugins inside their port, as stores live inside the cache subsystem of established platforms, addable without touching the port, including adapters written outside the repository; and folders named after what they are.
- Options considered: sibling crates per adapter; one crate per port with every adapter behind features; sub-crates inside the port's folder, each registering itself, with a generated registry in the server (chosen).
- Decision:
  - Folders drop the project prefix: `crates/common`, `crates/db`, `crates/cache`, `crates/server`… Package names keep it (`kampus-cache`), so imports stay unambiguous.
  - An adapter lives in `crates/<port>/adapters/<name>`, is named `<port package>-<name>` and exposes `ADAPTER`: its name, the URL schemes it serves and how it opens. The port's `open` chooses the adapter by the scheme of the configured URL.
  - `kampus-server` depends on every adapter and lists them in `crates/server/src/wiring/adapters.rs`; both are written by `tools/repo-check --fix` from the adapter folders and from `crates/server/adapters.toml`, which declares adapters published outside the repository (a git or registry source). CI fails when the registry is stale, so an adapter dropped into its folder is never forgotten.
  - Only `kampus-server` depends on adapters; modules depend on ports (dependency rule 5), checked by `tools/repo-check`.
  - Adapters shipped inside WebAssembly plugins (DEC-003) register through the plugin host when it exists; they pass the same conformance suites.
- Consequences: Adding a backend is a folder or a declaration plus a green conformance suite; the configuration chooses among the adapters of the build by URL alone.
- Affected artifacts: `crates/*`, `tools/repo-check`, `crates/server`, the infrastructure ports design, the platform developer guides.
