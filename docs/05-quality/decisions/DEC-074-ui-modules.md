---
id: DEC-074
status: accepted
---

# DEC-074 UI modules with a manifest, typed extension points and an error boundary each

- Date: 2026-11-18
- Context: DEC-004 makes the client a shell loading UI modules. Without a contract for what a module may add and where, modules reach into each other's pages and one failure breaks the screen.
- Options considered: modules patching the shell; a fixed set of pages per module; a manifest of routes and contributions into typed extension points (chosen).
- Decision:
  - Every module and plugin ships at most one UI module with a `kampus-ui.json` manifest: routes, contributions, strings, client events.
  - Extension points (regions, menus, tabs, actions, form sections, settings panels, activity views) have typed props and events.
  - UI modules load on demand through the import map; each extension point is an error boundary.
  - Contributions follow module state and permissions; every route and contribution declares a help topic.
- Consequences: Plugins add or replace screens without forking the client; a failing module never takes a page down.
- Affected artifacts: `web/`, `@kampus/sdk`, the plugin SDK.
