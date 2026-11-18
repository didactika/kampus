---
id: DEC-071
status: accepted
---

# DEC-071 Layouts, menus and themes as validated data, with accessible design tokens

- Date: 2026-11-18
- Context: Page arrangements tied to page-type patterns, menus edited in text settings and themes that need code or a fork make customisation fragile and slow.
- Options considered: themes with templates and code; per-page settings; layouts, menus and design tokens as data per scope (chosen).
- Decision:
  - Pages have declared kinds with regions; a layout per page kind and scope places widgets with their settings.
  - Menus are trees of items with targets and permissions.
  - Themes are design tokens validated against a schema and WCAG 2.2 AA contrast, applied as CSS custom properties.
  - Unknown or disabled widgets are skipped, never break a page.
- Consequences: Campuses and teachers rearrange pages and brand them without code; the web client (DEC-004) renders all of it from data.
- Affected artifacts: `crates/customise`, the web client.
