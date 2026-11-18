---
id: DEC-067
status: accepted
---

# DEC-067 Signed plugin packages with a declarative manifest and a guarded lifecycle

- Date: 2026-11-18
- Context: Plugins installed by copying files, with contributions discovered by naming conventions and upgrades that run arbitrary code, break campuses and cannot be trusted.
- Options considered: files copied into the code tree; packages without signatures; signed packages with a manifest and a host-driven lifecycle (chosen).
- Decision:
  - A plugin is a signed package: a WebAssembly component plus `kampus-plugin.toml`, with a digest and an SDK range.
  - The manifest declares the type, the capabilities and every contribution of the `Module` contract (DEC-015).
  - The host drives install, grant, migrate, enable, upgrade, disable and remove; a failed step leaves the previous state.
  - Removing runs the privacy handler and shows content that referenced the plugin as unavailable.
- Consequences: Installing a plugin is a reviewed, reversible operation; the handbook references include plugin contributions.
- Affected artifacts: `crates/plugin-host`, the plugin SDK, `kampus plugin` commands.
