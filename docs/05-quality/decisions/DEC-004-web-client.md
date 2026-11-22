---
id: DEC-004
status: accepted
---

# DEC-004 A modern, modular web client

- Date: 2026-11-17
- Context: The owner wants a modern, modular front end, rather than server-rendered pages with per-page scripts, where every part is customisable and plugins bring their own interface. Business logic must stay in Rust behind the API.
- Options considered: a Rust full-stack client (Leptos); one monolithic TypeScript single-page application; a modular client: an application shell that loads UI modules at runtime.
- Decision:
  - **Shell**: SvelteKit with Svelte 5 (runes) and TypeScript, server-side rendering for first paint and SEO, client navigation afterwards.
  - **UI modules**: every Kampus module and every plugin ships its interface as an independent ES module package (pages, widgets, settings panels) registered in a UI manifest and loaded on demand through import maps. The shell defines extension points (regions, menus, page slots, form sections) that modules fill.
  - **Design system**: framework-agnostic Web Components with W3C design tokens, so plugin UIs written with any framework look and behave the same, and themes change tokens, not code.
  - **Data**: a typed API client generated from the OpenAPI document; real-time updates over WebSockets where a module needs them.
  - **Quality**: WCAG 2.2 AA components, internationalisation and right-to-left from the start, Playwright end-to-end and accessibility tests, Vite builds.
- Consequences: Plugins can add or replace screens without forking the client; layouts and themes are data (DEC-008). The UI module contract (manifest, extension points, events) becomes part of the plugin SDK.
- Affected artifacts: `web/`, the plugin SDK, the customisation framework, Q-002 (resolved by this decision when accepted).
- Superseded in part by: DEC-089. The shell is Next.js with React, the design system is built on MUI, and UI modules are built into the client.
- Refined by: DEC-074 (UI modules), DEC-075 (design system) and DEC-076 (client data); design in the [web client](../../03-design/web/README.md).
