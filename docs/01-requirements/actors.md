---
id: ACT
status: draft
origin: requirements
sourceOfTruth: model
---

# Actors

Assumed from the owner's requirements and the usual roles of a learning platform; confirmed when each module is designed. In Kampus an actor is a role assigned in a scope (campus, category, course, section, activity), not a property of the user.

| Actor | Type | Goal | Related use cases |
| --- | --- | --- | --- |
| Visitor | Primary | Sign up or sign in, browse the public catalogue | To design |
| Learner | Primary | Take courses, do activities, see grades and progress | To design |
| Teacher | Primary | Run a course: content, activities, grading, communication | To design |
| Non-editing teacher | Primary | Grade and support learners without changing the course | To design |
| Section leader | Primary | Manage one section of a course: its content, members, grading and visibility, with roles given at section level (DEC-007) | To design |
| Course creator | Primary | Create courses in the categories they are allowed to | To design |
| Manager | Primary | Manage categories, courses, users and enrolments in their scope | To design |
| Campus administrator | Primary | Configure the platform, plugins, authentication, security and integrations | To design |
| Tenant administrator | Primary | Administer one tenant of a multi-tenant installation (extra) | To design |
| Guest | Primary | Read courses open to guests without enrolling | To design |
| Mentor | Primary | Follow the progress of the learners assigned to them (parent, tutor, supervisor) | To design |
| Operator | Primary | Install, configure (database, file storage, cache), upgrade, back up and monitor Kampus (DEC-009) | To design |
| Integrator | Primary | Connect other systems through the API, webhooks and LTI | To design |
| Plugin developer | Primary | Build Kampus plugins with the SDK, or convert existing plugins with `moodle2kampus` | To design |
| Identity provider | Secondary | Authenticate users (OIDC, SAML, LDAP) | To design |
| External system | Secondary | Integrate through the API, webhooks and LTI (student information systems, tools) | To design |
| Mail and notification services | Secondary | Deliver email, push and other notifications | To design |
| AI provider | Secondary | Serve the AI tutor and pre-grading extras, under the campus configuration | To design |
| Scheduler | System | Run scheduled and ad hoc tasks (progress, notifications, cleanup) | To design |
