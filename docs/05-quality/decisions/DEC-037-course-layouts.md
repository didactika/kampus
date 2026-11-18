---
id: DEC-037
status: proposed
---

# DEC-037 Course layouts as plugins that only present

- Date: 2026-11-18
- Context: Courses are presented in different ways (topics, weeks, single activity, social), and campuses want their own presentations. Layouts that own data make switching risky.
- Options considered: a fixed set of formats in the core; layouts that store their own structure; layout plugins that present the shared structure and keep only typed settings (chosen).
- Decision:
  - A layout is a plugin (DEC-003) with a UI module (DEC-004) and a typed settings schema.
  - Layouts never own sections or course modules; switching keeps all content and remembers each layout's settings for the course.
  - If a layout is disabled, its courses fall back to the campus default layout.
- Consequences: Campuses can add presentations without risk to content; teachers can switch and switch back.
- Affected artifacts: `crates/kampus-courses`, the plugin SDK, `web/`.
