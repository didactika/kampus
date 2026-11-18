---
id: RQ
status: draft
origin: requirements
sourceOfTruth: model
---

# Requirements catalogue

Priorities: **Must** (tier 1, core), **Should** (tier 2, standard activities and services), **Could** (tier 3, Kampus extras). Origin: **owner** (stated by the owner, confirmed), **baseline** (expected of any learning platform; assumed until its module is designed), **extra** (proposed for Kampus; assumed). Use cases are written when each module is designed.

## Functional

### Platform

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-001 | The platform is written from scratch in Rust | Must | owner | Owner, 2026-11-17 | |
| RF-002 | The platform is modular: each functional area is a module with its own data and public API, and can be enabled or disabled | Must | owner | Owner, 2026-11-17 | |
| RF-003 | The platform offers the complete core of a learning platform: courses, enrolment, roles and permissions, activities, academic records, progress, communication and administration | Must | owner | Owner, 2026-11-17 | |
| RF-004 | The platform is highly customisable: behaviour, appearance and workflows can be changed by configuration and plugins | Must | owner | Owner, 2026-11-17 | |
| RF-005 | The platform adds useful features beyond the standard set (tier 3) | Could | owner | Owner, 2026-11-17 | |
| RF-006 | Everything is customisable: settings, terminology, custom fields, layouts and navigation, themes, rules and workflows, notification templates and features, layered by context (DEC-008) | Must | owner | Owner, 2026-11-17 | |
| RF-007 | Every module is designed for Kampus: it keeps what works in established practice, solves known limitations, and records each improvement as a decision and in the highlights register | Must | owner | Owner, 2026-11-17 | |
| RF-008 | The web client uses modern, modular front-end technology: each module and plugin ships its own UI module, loaded at runtime into extension points of the shell (DEC-004) | Must | owner | Owner, 2026-11-17 | |
| RF-009 | Documentation for every role, shipped with each release: guides for learners, teachers, section leaders, course creators, managers, mentors, administrators, operators, integrators, plugin developers and core contributors, a migration guide, generated references, and help inside the product with onboarding and guided tours (DEC-011) | Must | owner | Owner, 2026-11-17 | |

### Identity and access

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-010 | Users sign up, confirm their email, sign in and reset their password | Must | baseline | | UC-006 |
| RF-011 | Users can sign in with external identity providers (OIDC first, then SAML and LDAP) | Must | baseline | | UC-007 |
| RF-012 | Users can enable multi-factor authentication; administrators can require it per role | Must | baseline | | UC-008 |
| RF-013 | Administrators manage users: create, edit, suspend, delete, bulk upload | Must | baseline | | UC-009 |
| RF-014 | Users have a profile with standard and custom fields, preferences and a picture | Must | baseline | | UC-010 |
| RF-015 | System groups gather users at campus or category level for enrolment and reporting | Should | baseline | | |

### Permissions

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-020 | Permissions are granted to roles and evaluated in a hierarchy of scopes: tenant, campus, category, course, **section**, activity, widget, user (the section level is new in Kampus, DEC-007) | Must | owner | Owner, 2026-11-17 | UC-001 |
| RF-024 | Roles, permission overrides, settings, visibility and availability rules can be set on a section and are inherited by its activities | Must | owner | Owner, 2026-11-17 | UC-002 |
| RF-021 | Administrators define roles, archetypes, the scopes where a role can be assigned, and overrides per scope | Must | baseline | | UC-004 |
| RF-022 | Every operation checks the permission of the acting user in the right scope | Must | baseline | | UC-003 |
| RF-023 | Users can see why they have or lack a permission (permission explanation) | Should | extra | | UC-005 |

### Courses

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-030 | Courses are organised in nested categories | Must | baseline | | UC-011 |
| RF-031 | Teachers create and configure courses: format, sections, visibility, dates | Must | baseline | | UC-012 |
| RF-032 | Course content is organised in sections holding activities and resources | Must | baseline | | UC-013 |
| RF-033 | Groups and group sets split learners inside a course | Must | baseline | | UC-014 |
| RF-034 | Course formats are plugins (topics, weekly, single activity and more) | Should | baseline | | UC-015 |

### Enrolment

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-040 | Users are enrolled in courses through enrolment methods (manual, self, system group, guest, and plugins) | Must | baseline | | UC-016, UC-019 |
| RF-041 | Enrolments have a status, start and end dates, and a role in the course | Must | baseline | | UC-018, UC-020 |
| RF-042 | Enrolment methods can require a key, an approval or a payment (payment through plugins) | Should | baseline | | UC-017 |

### Activities

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-050 | Activities and resources are plugins registered in an activity framework with common settings (visibility, availability, progress, grading, groups) | Must | baseline | | UC-021, UC-025 |
| RF-051 | Availability conditions restrict access to activities and sections (dates, grades, progress, groups, profile fields) | Must | baseline | | UC-022 |
| RF-052 | The standard activities exist: page, URL, file, folder, label, assignment, forum, quiz with question bank, survey, choice | Should | baseline | | |
| RF-053 | Activity types support backup, restore and course reset | Should | baseline | | UC-024 |
| RF-054 | **Exceptions**: every activity with dates, attempts, time limits or submissions accepts exceptions per user and per group (other dates, more attempts, more time), through one model, screen and API in the activity framework. (HL-002) | Must | owner | Owner, 2026-11-17 | UC-023 |

### Grades and progress

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-060 | Academic records per course hold grade records, categories and aggregation rules | Must | baseline | | UC-027 |
| RF-061 | Activities send grades to the academic records; teachers can override and lock them | Must | baseline | | UC-026, UC-028 |
| RF-062 | Learners see their grades and feedback; teachers see and export the grader report | Must | baseline | | UC-029 |
| RF-063 | Activity and course progress are tracked with configurable criteria | Must | baseline | | UC-030, UC-031, UC-032 |
| RF-064 | Achievements and skills recognise what learners accomplish | Should | baseline | | |

### Communication and calendar

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-070 | Users receive notifications through configurable channels (web, email, push) | Must | baseline | | |
| RF-071 | Users send direct and group messages | Should | baseline | | |
| RF-072 | A calendar shows campus, course, group and user events and activity dates | Must | baseline | | |

### Files, events and logs

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-080 | Files are stored once, referenced by scope and area, and served only with permission checks | Must | baseline | | |
| RF-081 | Every significant action is recorded as an event, logged and observable by modules and plugins | Must | baseline | | |
| RF-082 | Teachers and managers see logs and activity reports | Should | baseline | | |

### Administration

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-090 | Administrators configure the platform through settings, with defaults and per-module settings | Must | baseline | | |
| RF-091 | The platform is internationalised: language packs, per-user language, right-to-left support | Must | baseline | | |
| RF-092 | Scheduled and ad hoc tasks run in the background and can be monitored | Must | baseline | | |
| RF-093 | Users can export and delete their personal data; administrators manage data requests and retention | Must | baseline | | |

### Plugins and integration

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-100 | Plugins of these types (activity, widget, authentication, enrolment, report, question type, course format, theme, local) can be installed, upgraded, enabled and removed | Must | owner | Owner, 2026-11-17 | |
| RF-101 | A tool converts a Moodle plugin into a Kampus plugin quickly, reporting what needs review | Must | owner | Owner, 2026-11-17 | |
| RF-102 | Plugins run sandboxed with only the permissions they declare | Must | extra | | |
| RF-103 | External systems integrate through a documented API with tokens and scopes | Must | baseline | | |
| RF-104 | Webhooks notify external systems of events | Could | extra | | |
| RF-105 | LTI 1.3 lets Kampus launch external tools and be launched as a tool | Should | baseline | | |
| RF-106 | SCORM, H5P and xAPI content can be played and tracked | Should | baseline | | |

### Extras

| ID | Requirement | Priority | Origin | Confirmed by | Use cases |
| --- | --- | --- | --- | --- | --- |
| RF-110 | One installation serves several isolated tenants, each with its own branding and settings | Could | extra | | |
| RF-111 | An importer brings Moodle sites and `.mbz` course backups into Kampus | Could | extra | | |
| RF-112 | An AI tutor answers learners about course content, and AI pre-grading proposes grades that a teacher approves | Could | extra | | |
| RF-113 | Learning paths chain courses and activities with prerequisites across courses | Could | extra | | |
| RF-114 | Real-time presence and collaborative editing in course content and activities | Could | extra | | |
| RF-115 | Themes are built from design tokens and can be changed without code | Could | extra | | |
| RF-116 | Learning analytics are recorded as xAPI statements in a learning record store | Could | extra | | |
| RF-117 | The mobile experience works offline and synchronises when connected | Could | extra | | |
| RF-118 | Features can be switched on per campus, tenant or course with feature flags | Could | extra | | |
| RF-119 | After the initial release: book, lesson, glossary, wiki, database and workshop activities; tags, comments, notes and favourites; blogs, IMS content package import, BigBlueButton as a plugin, an accessibility checker (Q-003) | Could | owner | Owner, 2026-11-17 | |

## Non-functional

| ID | Quality attribute | Requirement | Measure | Priority |
| --- | --- | --- | --- | --- |
| RNF-001 | Security | OWASP ASVS 4 level 2 for every module; secure by default (authentication by default, ownership checks, strict contracts) | Security review per module; ASVS checklist | Must |
| RNF-002 | Privacy | GDPR: personal data declared per module, export and deletion, retention | Privacy tests per module | Must |
| RNF-003 | Accessibility | WCAG 2.2 AA for the web client | Automated scans plus manual audit per release | Must |
| RNF-004 | Performance | Fast page and API responses under load | p95 API latency under 200 ms at 1,000 concurrent users on reference hardware | Must |
| RNF-005 | Scalability | Horizontal scaling of the stateless server; state only in the database, file storage and cache ports | Load test with several server instances | Should |
| RNF-006 | Availability | Zero-downtime upgrades for minor versions | Rolling upgrade test | Should |
| RNF-007 | Maintainability | Modules respect their boundaries; no dependency cycles | Build fails on cycles (Cargo); architecture tests | Must |
| RNF-008 | Testability | Every use case has unit and end-to-end tests; every rule a test | Traceability table without gaps | Must |
| RNF-009 | Observability | Structured logs, metrics and traces (OpenTelemetry) | Dashboards and alerts in the reference deployment | Should |
| RNF-010 | Internationalisation | Every user-facing string translatable; dates, numbers and directions localised | No hard-coded strings (lint) | Must |
| RNF-011 | Extensibility | Plugins depend only on the versioned SDK; host API changes are backward compatible within a major version | SDK compatibility tests | Must |
| RNF-012 | Plugin isolation | A faulty plugin cannot crash the platform or read data it was not granted | Sandbox and fuzz tests | Must |
| RNF-013 | Portability | Runs as containers on Linux; one-command local setup | Compose file and Helm chart | Must |
| RNF-014 | Supply chain | Locked dependencies, audits in CI, signed releases | `cargo audit`, `cargo deny`, npm audit in CI | Must |
| RNF-015 | Code provenance | All code is original or comes from a declared dependency with an allowed licence (DEC-010) | Licence check in CI; review checklist per pull request | Must |
| RNF-016 | Database portability | Runs on several relational databases behind the database port: PostgreSQL first, then MySQL and MariaDB, SQLite, SQL Server and Oracle (DEC-009) | Every module test passes against every supported database in CI | Must (PostgreSQL), Should (others) |
| RNF-017 | File storage portability | File storage behind the storage port: local disk and S3-compatible first, then Azure Blob, Google Cloud Storage, WebDAV and SFTP; files can move between backends without downtime | Storage conformance suite per adapter; migration test | Must (local, S3), Should (others) |
| RNF-018 | Cache portability | Two-level cache behind the cache port: in-process and Redis or Valkey first, then Memcached; invalidation across nodes through the event bus | Cache conformance suite; multi-node invalidation test | Must (in-process, Redis), Should (others) |
| RNF-019 | Infrastructure ports | Sessions, task queue, search and mail are ports with interchangeable adapters; modules depend only on ports | Architecture test: no module depends on an adapter crate | Must |
| RNF-020 | Documentation quality | The handbook matches the release it ships with: every use case documented for the roles it affects, every screen linked to help, references generated, screenshots taken by the end-to-end tests, examples compiled, accessible and translatable | Documentation checks in CI (links, examples, help topics without a page); review per use case | Must |
| RNF-021 | Data integrity | A truly relational, normalised data model enforced by the database: real foreign keys, no polymorphic references, proper types, natural keys unique, invariants as checks, one schema per module, indexes from the queries of the use cases (DEC-014) | Schema linter in CI; query-plan tests of key use cases; no orphan or duplicate rows possible | Must |

## Business rules

Rules live in the domain model dictionary and are decided when each module is designed.
