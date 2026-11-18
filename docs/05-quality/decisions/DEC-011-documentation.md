---
id: DEC-011
status: accepted
---

# DEC-011 Documentation for every role, built with the product

- Date: 2026-11-17
- Context: The owner requires documentation for developers, learners, teachers and every other role, so that Kampus is easy to adopt and use from its first release. Documentation kept in a separate wiki tends to fall out of date with the version a site runs, ignores the site's own terminology, and is not linked from the screens.
- Options considered:
  - A wiki written after the release.
  - Documentation written by a separate team.
  - Documentation as code, produced with each use case and tested like code (chosen).
- Decision:
  - **Two kinds of documentation.**
    - `docs/` stays the engineering specification: requirements, design and decisions.
    - The **Kampus handbook** (`handbook/`) is the product documentation. It is a static site built with Starlight (Astro): searchable, translatable, versioned per release and accessible (WCAG 2.2 AA).
  - **One guide per audience.**
    - **Learner:** courses, activities, grades, progress, messages, calendar, profile.
    - **Teacher and non-editing teacher:** building courses and sections, activities, grading, groups, communication, reports.
    - **Section leader:** what can be managed in a section (DEC-007).
    - **Course creator and manager:** categories, courses, users, enrolment, system groups, roles.
    - **Mentor:** following learners.
    - **Site and tenant administrator:** settings, customisation (DEC-008), authentication, plugins, security, privacy.
    - **Operator:** installation, choosing and configuring the database, file storage and cache backends (DEC-009), upgrades, backup, monitoring, scaling.
    - **Integrator:** API, tokens, webhooks, LTI.
    - **Plugin developer:** SDK, WIT interfaces, UI modules, `moodle2kampus`.
    - **Core contributor:** architecture, modules, conventions.
    - **Migrating to Kampus:** how to move a site with the importer and its plugins with `moodle2kampus`.
  - **Generated references, never written by hand:**
    - the API reference, from OpenAPI;
    - the crate documentation, from rustdoc;
    - the SDK reference, from WIT;
    - the settings, permissions and events references, from the registries in the code.
  - **Help inside the product.**
    - Every screen of a UI module declares its help topics, which open the matching handbook page in context.
    - First-run onboarding and guided tours for each role, empty states that explain the next step, and tooltips.
    - Help shown inside the product uses the campus's own terminology and branding (DEC-008).
  - **Documentation is part of done.**
    - A use case is not done until the guides of the roles it affects are updated.
    - Screenshots are produced by the end-to-end tests (Playwright), so they always match the release.
    - Code examples are compiled and run as tests.
    - Links are checked in CI.
- Consequences:
  - More work per use case, but every release ships complete and current documentation.
  - The use cases and wireframes of the design are the source of the user guides.
  - The handbook is translated with the same internationalisation process as the product (RNF-010).
- Affected artifacts: RF-009, RNF-020, `handbook/`, the definition of done.
- Refined by: DEC-077 (help topics and tours) and DEC-078 (handbook structure) and DEC-084 (its own repository); design in the [handbook](../../03-design/handbook/README.md).
