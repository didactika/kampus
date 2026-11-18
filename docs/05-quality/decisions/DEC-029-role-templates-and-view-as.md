---
id: DEC-029
status: accepted
---

# DEC-029 Versioned role templates, and read-only "view as"

- Date: 2026-11-18
- Context: Roles start from defaults that improve over releases, and campuses customise them; teachers and managers need to see what others see without acting as them.
- Options considered: defaults hard-coded per release; free-form roles without templates; versioned templates with previewed resets, and a read-only view-as session (chosen).
- Decision:
  - Every role is created from a template version; a reset shows the differences and applies only what was previewed; an unknown template is an error.
  - New template versions ship with releases and never change a campus silently.
  - "View as" uses only the chosen role, read permissions only, inside the scope and below; it ends with the session or on leaving the scope and is audited.
- Consequences: Campuses keep their customisations across upgrades and adopt new defaults when they decide; viewing as can never be used to act in someone else's name.
- Affected artifacts: `crates/kampus-access`, the role screens, the release notes.
