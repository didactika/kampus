---
id: DEC-003
status: accepted
---

# DEC-003 Plugins: native modules and WebAssembly components

- Date: 2026-11-17
- Context: Kampus needs plugin types (activities, widgets, authentication, enrolment, reports, question types, themes, local) that third parties can install safely, and a target for plugins converted by the migration tools (DEC-006).
- Options considered: compile-time crates only; dynamic libraries; embedded scripting; WebAssembly components with WIT interfaces.
- Decision: Two levels. First-party core modules are native crates. Installable plugins are WebAssembly components (Component Model, WIT interfaces) run with Wasmtime, with capability-based access to the host API (data of their own tables, permissions, events, files, strings, templates).
- Consequences: Plugins are sandboxed and portable across versions of the host API; the WIT interfaces become the plugin SDK and the target of `moodle2kampus`.
- Affected artifacts: plugin SDK, `crates/plugin-host`, `tools/moodle2kampus/`.
