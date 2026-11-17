---
id: DEC-007
status: accepted
---

# DEC-007 A section scope in the permission hierarchy

- Date: 2026-11-17
- Context: Learning platforms usually evaluate permissions in a hierarchy platform → category → course → activity → widget, with no level for course sections (the owner pointed this out). Sections are then only containers for activities, so a course cannot give a role, override a permission, apply a setting or restrict visibility for a whole section in one place; teachers work around it with groups, availability conditions on every activity, or separate courses.
- Options considered: the usual hierarchy without sections; emulate sections with groups; add a section scope level.
- Decision: Kampus adds a **section** scope between course and activity: campus → category (nested) → course → section (nested sections allowed) → activity → widget, plus user and tenant scopes. Roles can be assigned, permissions overridden, settings and customisations applied, and availability rules set at section level, and they are inherited by the activities of the section.
- Consequences: Section-level teachers, assistants and reviewers; per-section visibility, release dates, grading and progress rules; simpler course design. The permission check walks one more level, which the scope model and its cache handle. The importer and the converter map course-level role assignments to the course scope and leave the section scope empty.
- Affected artifacts: RF-020, RF-024, the permission model, the courses model, the activity framework, the customisation framework (DEC-008), the importer.
