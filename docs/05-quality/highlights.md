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
| HL-018 | Reliability | Optimistic versions on every change, idempotency keys on every creation, and limits enforced by the database | DEC-026 | planned | No lost edits, no double submissions, no limit crossed by two clicks at once. |
| HL-019 | Integration | A REST API described by OpenAPI, with cursor pagination, problem details and additive versioning kept stable across releases | DEC-025 | planned | Integrations that keep working release after release. |
| HL-020 | Permissions | Roles granted for a period end on time; roles given by enrolment or system groups are owned by them and never remove a role given by hand | DEC-028 | planned | Temporary assistants and reviewers, without anyone remembering to remove them. |
| HL-021 | Permissions | Role templates are versioned; resetting a role shows the differences first, and upgrades never change a campus silently | DEC-029 | planned | Your role customisations survive every upgrade. |
| HL-022 | Permissions | A scope can be frozen: everything in it becomes read-only for everyone until it is unfrozen | RN-011 | planned | Close a course or a term in one click, without losing anything. |
| HL-023 | Identity | Passwords, passkeys and identity providers share one sign-in pipeline with throttling and second factors; external accounts link by issuer and subject | DEC-030 | planned | Every way in is equally safe. |
| HL-024 | Identity | Recovery asks for the second factor before changing any credential | DEC-032 | planned | Controlling someone's email is not enough to take over their account. |
| HL-025 | Identity | Suspending an account or changing its password ends every session and token at once | DEC-031 | planned | Access ends the moment you decide. |
| HL-026 | Privacy | Deleting a person erases their personal data after a grace period, while records keep a consistent anonymous reference | DEC-033 | planned | The right to be forgotten, without breaking anyone's grades. |
| HL-027 | Integration | API tokens are scoped, expire, are shown once and stored hashed | DEC-031 | planned | Integrations with exactly the access they need, for as long as they need it. |
| HL-028 | Courses | Hide or schedule any category, course or section; showing it again restores exactly what each part had | DEC-034 | planned | Hide a whole category or section and bring it back exactly as it was. |
| HL-029 | Courses | Sections nest to any depth and keep their identity when moved; links, permissions and groups follow them | DEC-035 | planned | Reorganise a course freely: links and settings follow each section. |
| HL-030 | Courses | Archiving a course freezes it read-only; deletion is a job with a grace period that removes everything it owns | DEC-036 | planned | Close a term in one click and keep it as a record. |
| HL-031 | Courses | Groups for one section, and group mode per course, section or activity | DEC-038 | planned | Teams where you need them, and only there. |
| HL-032 | Enrolment | Leaving a course and coming back never loses grades or progress | DEC-043 | planned | Come back to a course exactly where you left it. |
| HL-033 | Enrolment | Capacities hold under any load; self-enrolment keys are hashed, never hinted, and throttled | DEC-041 | planned | Fair places in every course, and keys nobody can guess. |
| HL-034 | Enrolment | Courses linked to programmes (system groups) or to other courses stay in sync, without loops | DEC-042 | planned | Enrol a whole programme once; every course follows. |
| HL-035 | Enrolment | Each way of joining owns the roles it gives, so removing one never removes another | DEC-040 | planned | No more surprises when someone leaves one of several courses or groups. |
| HL-036 | Activities | Exceptions (dates, attempts, time) for any activity, with the same answer in the activity, the calendar and notifications | DEC-046 | planned | Extensions and special arrangements for any activity. |
| HL-037 | Activities | Access conditions that never break silently: what they depend on cannot disappear unnoticed | DEC-045 | planned | Learning paths that keep working when the course changes. |
| HL-038 | Activities | A recycle bin for activities and sections, with restore and visible failures | DEC-047 | planned | Undo a deletion, even weeks later. |
| HL-039 | Records | Full history of every grade, and totals that never leak hidden grades | DEC-048, DEC-049 | planned | Every grade explained, and hidden grades stay hidden everywhere. |
| HL-040 | Records | Aggregation methods with worked examples, and calculation behaviour versioned per course | DEC-050 | planned | Totals teachers can predict, unchanged by upgrades. |
| HL-041 | Progress | Progress always current, revocable when criteria change, and never erasing what teachers decided | DEC-051 | planned | Progress you can trust, and explain. |
| HL-042 | Files | Each file stored once, readable exactly when what it belongs to is readable, served by expiring links | DEC-053 | planned | No orphaned files and no leaked downloads. |
| HL-043 | Files | One upload path with the same limits, scanning and quotas for every client | DEC-054 | planned | Clear errors before uploading too much. |
| HL-044 | Events | Typed events delivered reliably, with failures visible and retryable, and an audit log that cannot be altered | DEC-055 | planned | Know who did what, and trust integrations. |
| HL-045 | Notifications | Preferences per notification and channel, required notices that cannot be lost, and texts in the campus language | DEC-056 | planned | The right notices, on the right channel. |
| HL-046 | Messaging | One conversation per pair, campus rules on who may write, and deleted messages that really go away | DEC-057 | planned | Private messaging you can trust. |
| HL-047 | Calendar | Activity dates and extensions always current in the calendar, repeating events and subscriptions without duplicates | DEC-058 | planned | One calendar that is always right. |
| HL-048 | Settings | Every setting typed and validated, showing where its value comes from, with locks and a full history | DEC-059 | planned | Know why a setting has its value, and who changed it. |
| HL-049 | Tasks | Background work that never gets stuck, never runs twice and never fails without an alert | DEC-060 | planned | Scheduled tasks you can trust and monitor. |
| HL-050 | Privacy | Export and deletion covering every module and plugin by construction, with legal holds and retention per data category | DEC-061 | planned | GDPR requests answered completely, on time. |
| HL-051 | Languages | Translations with correct plurals, customisations that survive updates and missing strings made visible | DEC-062 | planned | Every language done properly, including right-to-left. |
| HL-052 | Integrations | A generated API document, least-privilege tokens, OAuth for third-party apps and rate limits | DEC-063 | planned | Safe integrations that match the documentation. |
| HL-053 | Integrations | Signed, reliable webhooks filtered by permissions | DEC-064 | planned | Push events to other systems without polling. |
| HL-054 | Infrastructure | One schema for every supported database, with migrations tested on each | DEC-065 | planned | Run on the database your organisation already uses. |
| HL-055 | Infrastructure | Every storage, cache and database backend proven by the same conformance suite | DEC-066 | planned | Swap backends without surprises. |
