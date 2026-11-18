---
id: DOM
status: draft
origin: requirements
sourceOfTruth: model
---

# Domain model

## Purpose

The business concepts of Kampus, module by module. Each module has its own document with entities, rules, flows, lifecycles, customisation points and contracts.

| Module | Document | Status |
| --- | --- | --- |
| `kampus-access`: scopes, roles, permissions | [access.md](./access.md) | accepted |
| `kampus-identity` and `kampus-users`: accounts, sign-in, profiles, system groups | [identity.md](./identity.md) | accepted |
| `kampus-courses`: categories, courses, sections, groups | [courses.md](./courses.md) | accepted |
| `kampus-enrol`: methods, enrolments, requests | [enrolment.md](./enrolment.md) | accepted |
| `kampus-activities`: course modules, access conditions, exceptions | [activities.md](./activities.md) | proposed |
| `kampus-records`: academic records | [records.md](./records.md) | proposed |
| `kampus-progress`: activity and course progress | [progress.md](./progress.md) | proposed |

## Hierarchy of entities

The vertical position of an entity expresses its importance to the project objective. It is not UML inheritance or composition.

| Level | Meaning | Entities | Why they are here |
| --- | --- | --- | --- |
| 1 | Core: the project cannot exist without them | Scope, Permission, Role, Grant; Account, Session, Profile; Course category, Course, Section, Group; Method instance, Enrolment; Course module, Exception; Grade record, Grade entry, Grade; Activity progress, Course progress | Every operation is a permission check in a scope |
| 2 | Structural: depend directly on the core | Role permission, Permission override, Permission value; Credential, External identity, Second factor, API token, Profile field, System group, Course layout, Group set, Group membership, Enrolment method, Enrolment grant, Enrolment request, Linked source, Activity type, Access condition, Grade category, Progress rule, Course criterion | Define what roles allow, and where |
| 3 | Detail: qualify or specialise level 2 | Risk, Role template, Role allowance, View-as session | Constrain and template roles |
| 4 | Support: catalogues, audit, configuration | Permission decision | Explanations |

## Diagrams

![Domain model](./domain-model.svg)

![Simplified domain model](./domain-model-simple.svg)

![Access](./access.svg)

![Identity](./identity.svg)

![Courses](./courses.svg)

![Enrolment](./enrolment.svg)

![Activities](./activities.svg)

![Records](./records.svg)

![Progress](./progress.svg)

## Entity dictionary

Per module: [access](./access.md#entities), [identity](./identity.md#entities), [courses](./courses.md#entities), [enrolment](./enrolment.md#entities), [activities](./activities.md#entities), [records](./records.md#entities), [progress](./progress.md#entities).

## Business rules

Per module: [access](./access.md#business-rules) (RN-001 to RN-021), [identity](./identity.md#business-rules) (RN-022 to RN-039), [courses](./courses.md#business-rules) (RN-040 to RN-059), [enrolment](./enrolment.md#business-rules) (RN-060 to RN-074), [activities](./activities.md#business-rules) (RN-075 to RN-092), [records](./records.md#business-rules) (RN-095 to RN-110), [progress](./progress.md#business-rules) (RN-115 to RN-125).

## Lifecycles

State diagrams live in [entity-states](./entity-states/entity-states.svg).
