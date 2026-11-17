---
id: DEC-008
status: accepted
---

# DEC-008 Everything customisable, layered by scope

- Date: 2026-11-17
- Context: The owner requires that everything in Kampus can be customised. In most learning platforms, settings, themes and plugins cover part of it, but behaviour, wording, forms and layouts need code or a theme fork to change.
- Options considered: settings per module only; themes plus plugins; a platform-wide customisation layer resolved through the scope hierarchy.
- Decision: A customisation framework in the core, resolved through the scope hierarchy (platform → tenant → category → course → section → activity → user), where each lower level may override what the level above allows:
  - **Settings**: every module declares typed settings with defaults, the levels where each can be overridden, and who may change them.
  - **Terminology**: every user-facing term (course, section, teacher, grade…) can be renamed per tenant, category or course, on top of language packs.
  - **Custom fields**: any entity (user, course, section, activity, enrolment, grade record…) can get custom fields with types, validation and visibility.
  - **Layouts and navigation**: pages are made of regions and widgets that administrators and teachers can arrange; navigation menus are data.
  - **Themes**: design tokens (colour, typography, spacing, radius, motion) editable without code, per tenant and per course.
  - **Rules and workflows**: configurable rules (when an event happens and conditions hold, do actions) for enrolment, progress, notifications and grading, without plugins.
  - **Notifications and templates**: every message and email template editable per level.
  - **Feature flags**: modules and features switched per level.
- Consequences: One resolution engine with caching, audit of who changed what, and import and export of customisations between campuses. Plugins use the same framework for their settings, terms and widgets.
- Affected artifacts: RF-004, RF-006, the settings module, the web client, every module's design.
