---
id: DEC-068
status: accepted
---

# DEC-068 WIT host interfaces granted as capabilities, with resource limits per call

- Date: 2026-11-18
- Context: DEC-003 sandboxes plugins in WebAssembly. Without a precise list of what a plugin may call and limits on what a call may consume, the sandbox protects nothing (RNF-012).
- Options considered: full host access inside the sandbox; one permission for everything; one WIT interface per concern granted as a capability (chosen).
- Decision:
  - Each plugin type is a WIT world; host interfaces are imports grouped by concern (context, permissions, own storage, read models, events, jobs, files, strings, settings, HTTP to declared hosts, logs).
  - Capabilities are granted by an administrator per version; an ungranted import fails.
  - Read models apply the caller's permissions; there is no raw SQL, file system or free network.
  - Every call has limits on fuel, memory and wall time; repeated failures disable the plugin.
  - Interfaces change additively within a major SDK version.
- Consequences: A faulty or hostile plugin can only fail its own calls; plugins keep working across minor host releases (RNF-011).
- Affected artifacts: `crates/kampus-plugin-host`, `kampus-sdk`, `@kampus/sdk`.
