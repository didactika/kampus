---
id: DEC-089
status: accepted
---

# DEC-089 The web client in React with Next.js and MUI, themed by Didactika

- Date: 2026-11-22
- Context: The owner wants the web client built on a widely known stack: React, Next.js and MUI (Material UI), always in TypeScript and well ordered. Didactika's plugins already use React and TypeScript, so one stack serves all of them. DEC-004 chose SvelteKit, and DEC-075 chose framework-agnostic Web Components. Themes per scope (RN-217) and Didactika's identity (DEC-013) must keep working.
- Options considered:
  - Keep SvelteKit and Web Components.
  - Use React with Material Web components.
  - Use React with Next.js and MUI (chosen).
- Decision:
  - **Stack.** The web client is a Next.js application (App Router, React Server Components, TypeScript in strict mode) in `web/`, served by its own Node server (`output: "standalone"`).
  - **Design system.** `@kampus/ui` is a React package built on MUI. Its theme is made from Didactika's tokens (DEC-013), which prevail over MUI's defaults. It holds the components of Didactika's design: thin layers over MUI that fix what each element means (one main action per area, danger always asks first, the toolbar, filters, table and pager of lists). Icons are Material Symbols by name, through a registry a theme can override.
  - **Themes per scope.** The theme of a scope keeps coming from the server as colour roles grown from seed colours. The client turns them into the MUI theme with CSS variables, so a scope changes the look with no code and no rebuild, in light and dark.
  - **UI modules.** Each Kampus module and plugin keeps one UI module next to its crate (`crates/<module>/web`), with its manifest. UI modules are **built into the client**: a registry generated from the installed modules lists their routes and contributions, the same way the server registers its modules (DEC-087), so React and MUI exist once. Installing or removing a plugin rebuilds the client image.
  - **Versions.** React, Next.js, MUI, Material Symbols and the tools of the client follow their latest stable release, pinned exactly and updated on purpose.
  - **Tests.** Components are tested in a real browser with axe (WCAG 2.2 AA in both schemes). Pages are tested end to end with Playwright.
- Consequences:
  - The Web Components of the first `@kampus/ui` are replaced by React components.
  - Plugins write their interface in React with `@kampus/ui`; a plugin's interface takes effect when the client is rebuilt, not at run time.
  - The extension points, the manifest, the error boundary per extension point (DEC-074) and the generated API client (DEC-076) stay as they were.
- Affected artifacts: the [Kampus design system](../../03-design/web/design-system.md), `web/`, `web/ui`, `web/sdk`, `crates/<module>/web`, the plugin SDK, DEC-004, DEC-074, DEC-075.
