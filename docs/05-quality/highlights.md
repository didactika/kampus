---
id: HL
status: draft
origin: requirements
sourceOfTruth: model
---

# Highlights

What makes Kampus stand out. This register is the source for the release notes, the migration guide and the announcements of Kampus.

**Rules**
- Every design choice that solves a known limitation of learning platforms gets an entry here, in the same pull request as the requirement or decision behind it.
- Status follows the work:
  - **planned:** a requirement or decision exists;
  - **designed:** the module design is approved;
  - **built:** in a release;
  - **announced:** in the release notes.
- The **pitch** column is one sentence for the public, in the owner's language.

| ID | Area | Kampus | Ref | Status | Pitch |
| --- | --- | --- | --- | --- | --- |
| HL-001 | Permissions | A section scope with its own roles, permission overrides, settings and visibility, inherited by its course modules | DEC-007, RF-020, RF-024 | planned | Give a teacher, an assistant or a reviewer exactly one section of a course. |
| HL-002 | Activities | **Exceptions** are part of the activity framework. Every activity with dates, attempts, time limits or submissions supports exceptions per user and per group, with one model, one screen and one API | RF-054 | planned | Extensions and special arrangements for any activity. |
| HL-003 | Data | A relational model enforced by the database: real foreign keys, proper types, constraints, one schema per module, and a schema linter in CI | DEC-014, RNF-021 | planned | Your data stays consistent: the database itself guarantees it. |
| HL-004 | Customisation | Everything is customisable per scope: settings, terminology, custom fields, layouts, navigation, themes, rules and templates, per campus, category, course or section | DEC-008, RF-006 | planned | Shape Kampus to each faculty, course or section without code. |
| HL-005 | Infrastructure | Every port has a conformance suite, and every module test runs on every supported database in CI. Files can move between storage backends without downtime | DEC-009, RNF-016 to RNF-019 | planned | Run on the database, storage and cache you already have. |
| HL-006 | Plugins | WebAssembly plugins run sandboxed with the permissions they declare. A faulty plugin cannot break the platform or read data it was not granted | DEC-003, RNF-012 | planned | Install extensions without risking your platform. |
| HL-007 | Migration | `moodle2kampus` converts existing plugins: deterministic for the declarative parts, assisted for the logic, with a report | DEC-006, RF-101 | planned | Bring your plugins with you. |
| HL-008 | Interface | A modern, modular web client: every module and plugin ships its own UI module, built on a design system of Web Components and design tokens | DEC-004, RF-008 | planned | A fast, modern interface that every plugin can extend. |
| HL-009 | Branding | Brand colours, logo and icon are design tokens edited in settings with a live preview. Changes that break WCAG 2.2 AA contrast are blocked | DEC-013 | planned | Your brand in minutes, accessible by design. |
| HL-010 | Documentation | A handbook for every role, versioned with each release, with help on every screen and guided tours, using the campus's own terminology | DEC-011, RF-009 | planned | Help for every role, in the right place and in your own words. |
| HL-011 | Administration | Administration is a role with permissions in the campus scope. Every grant is a row with history, and no check is bypassed silently | RF-021, RF-022 | planned | Even administrators act within clear, audited permissions. |
| HL-012 | API | Every permission operation has an API endpoint: roles, overrides, allowances, reviews, explanations | RF-103 | planned | Manage roles and permissions from any system through the API. |
| HL-013 | Permissions | An explanation of every permission result: the roles, scopes and overrides that decided it | RF-023 | planned | Know exactly why someone can, or cannot, do something. |
| HL-014 | Permissions | "View as" any role in any scope, recorded in the audit log | RF-020 | planned | See any part of Kampus exactly as a student, assistant or reviewer sees it. |
| HL-015 | Reliability | Every change and its events are committed together through an outbox; listeners, mail and notifications run after commit with retries and a visible failed state | DEC-016 | planned | Nothing a learner or teacher does is lost, and no failure goes unnoticed. |
| HL-016 | Security | Permissions are checked inside every service operation, against the scope of the object, for the web client, the API and plugins alike; lists are filtered before paging | DEC-017 | planned | One permission rule everywhere: no back door through the API or a plugin. |
| HL-017 | Operations | One image with server, worker and scheduler roles, stateless and scaling horizontally, observable with OpenTelemetry, with an audit log apart | DEC-019, DEC-020 | planned | From one small server to a cluster with the same image, and you can see everything it does. |
