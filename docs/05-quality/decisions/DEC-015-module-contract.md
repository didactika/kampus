---
id: DEC-015
status: accepted
---

# DEC-015 Module contract: service APIs, events and declared contributions

- Date: 2026-11-18
- Context: DEC-002 makes every module a crate. The modules still need one way to talk to each other and one way to plug into the platform (permissions, settings, events, jobs, privacy, routes, UI modules, help), or each module would invent its own and the boundaries would erode.
- Options considered: modules calling each other freely; a shared database with joins across modules; a central registry edited by hand; a `Module` contract that each crate implements and the server collects (chosen).
- Decision:
  - Every module exposes a **service API** (traits and types) and **domain events**, and owns its schema.
  - Modules call only modules of a lower level; upward communication is by events (dependency rules of the [architecture](../../03-design/architecture.md)).
  - Every module implements one `Module` trait from `kampus-common` that **declares** its permissions, settings, events, jobs, privacy data handlers, API routes, UI modules and help topics. `kampus-server` collects them at start-up; nothing is discovered by file names or naming conventions.
  - Plugins (DEC-003) declare the same contributions in their manifest.
- Consequences: one place to read what a module adds to the platform; the permissions, settings and events references of the handbook (DEC-011) are generated from these declarations; an architecture test checks the dependency rules on every build.
- Affected artifacts: `crates/common`, every module crate, `crates/server`, the plugin SDK.
