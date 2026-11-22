---
id: DEC-075
status: superseded
---

# DEC-075 A Web Components design system with W3C tokens and accessibility checked in CI

- Date: 2026-11-18
- Context: UI modules may use different frameworks; without shared components they look and behave differently and accessibility depends on each author.
- Options considered: one framework's component library; CSS only; framework-agnostic Web Components with design tokens (chosen).
- Decision:
  - `@kampus/ui` provides Web Components for every common control, themed only through W3C design tokens.
  - Components meet WCAG 2.2 AA (keyboard, focus, roles, contrast, reduced motion, labelled errors) and use logical CSS properties for right-to-left.
  - Accessibility scans, contrast checks and visual regression run in CI.
- Consequences: Every module looks and behaves the same in every theme; themes change tokens, never code.
- Affected artifacts: `web/ui`, every UI module.
- Superseded by: DEC-089. `@kampus/ui` is a React design system built on MUI and themed by Didactika. Its tokens, accessibility checks and right-to-left support remain.
