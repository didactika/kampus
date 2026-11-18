---
id: REQ
status: draft
origin: requirements
sourceOfTruth: model
---

# Requirements

## Index

- [Actors](./actors.md)
- [Requirements catalogue](./requirements.md)

## Use cases

| ID | Use case | Actor | Package | Requirement | Analysis | Design | Implementation | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| UC-001 | [Grant a role in a scope](./use-cases/grantRole/README.md) | Manager, Teacher | access | RF-020 | [card](../02-analysis/use-cases/grantRole/README.md) | [card](../03-design/use-cases/grantRole/README.md) | [card](../04-implementation/use-cases/grantRole/README.md) | accepted |
| UC-002 | [Override permissions in a scope](./use-cases/overridePermission/README.md) | Manager, Teacher | access | RF-024 | [card](../02-analysis/use-cases/overridePermission/README.md) | [card](../03-design/use-cases/overridePermission/README.md) | [card](../04-implementation/use-cases/overridePermission/README.md) | accepted |
| UC-003 | [Check a permission, with its explanation](./use-cases/checkPermission/README.md) | System | access | RF-022 | [card](../02-analysis/use-cases/checkPermission/README.md) | [card](../03-design/use-cases/checkPermission/README.md) | [card](../04-implementation/use-cases/checkPermission/README.md) | accepted |
| UC-004 | [Define a role from a template](./use-cases/defineRole/README.md) | Campus administrator | access | RF-021 | [card](../02-analysis/use-cases/defineRole/README.md) | [card](../03-design/use-cases/defineRole/README.md) | [card](../04-implementation/use-cases/defineRole/README.md) | accepted |
| UC-005 | [View a scope as another role](./use-cases/viewAsRole/README.md) | Teacher, Manager | access | RF-023 | [card](../02-analysis/use-cases/viewAsRole/README.md) | [card](../03-design/use-cases/viewAsRole/README.md) | [card](../04-implementation/use-cases/viewAsRole/README.md) | accepted |
| UC-006 | [Sign up and confirm an account](./use-cases/signUp/README.md) | Visitor | identity | RF-010 | [card](../02-analysis/use-cases/signUp/README.md) | [card](../03-design/use-cases/signUp/README.md) | [card](../04-implementation/use-cases/signUp/README.md) | accepted |
| UC-007 | [Sign in, or recover access](./use-cases/signIn/README.md) | Visitor | identity | RF-011 | [card](../02-analysis/use-cases/signIn/README.md) | [card](../03-design/use-cases/signIn/README.md) | [card](../04-implementation/use-cases/signIn/README.md) | accepted |
| UC-008 | [Set up a second factor](./use-cases/setUpSecondFactor/README.md) | Authenticated user | identity | RF-012 | [card](../02-analysis/use-cases/setUpSecondFactor/README.md) | [card](../03-design/use-cases/setUpSecondFactor/README.md) | [card](../04-implementation/use-cases/setUpSecondFactor/README.md) | accepted |
| UC-009 | [Manage the lifecycle of accounts](./use-cases/manageAccountLifecycle/README.md) | Manager | identity | RF-013 | [card](../02-analysis/use-cases/manageAccountLifecycle/README.md) | [card](../03-design/use-cases/manageAccountLifecycle/README.md) | [card](../04-implementation/use-cases/manageAccountLifecycle/README.md) | accepted |
| UC-010 | [Edit a profile](./use-cases/editProfile/README.md) | Authenticated user, Manager | users | RF-014 | [card](../02-analysis/use-cases/editProfile/README.md) | [card](../03-design/use-cases/editProfile/README.md) | [card](../04-implementation/use-cases/editProfile/README.md) | accepted |
| UC-011 | [Manage course categories](./use-cases/manageCategories/README.md) | Manager | courses | RF-030 | [card](../02-analysis/use-cases/manageCategories/README.md) | [card](../03-design/use-cases/manageCategories/README.md) | [card](../04-implementation/use-cases/manageCategories/README.md) | proposed |
| UC-012 | [Create and configure a course](./use-cases/createCourse/README.md) | Course creator, Manager | courses | RF-031 | [card](../02-analysis/use-cases/createCourse/README.md) | [card](../03-design/use-cases/createCourse/README.md) | [card](../04-implementation/use-cases/createCourse/README.md) | proposed |
| UC-013 | [Organise the sections of a course](./use-cases/organiseSections/README.md) | Teacher | courses | RF-032 | [card](../02-analysis/use-cases/organiseSections/README.md) | [card](../03-design/use-cases/organiseSections/README.md) | [card](../04-implementation/use-cases/organiseSections/README.md) | proposed |
| UC-014 | [Manage groups and group sets](./use-cases/manageGroups/README.md) | Teacher | courses | RF-033 | [card](../02-analysis/use-cases/manageGroups/README.md) | [card](../03-design/use-cases/manageGroups/README.md) | [card](../04-implementation/use-cases/manageGroups/README.md) | proposed |
| UC-015 | [Choose a course layout](./use-cases/chooseCourseLayout/README.md) | Teacher | courses | RF-034 | [card](../02-analysis/use-cases/chooseCourseLayout/README.md) | [card](../03-design/use-cases/chooseCourseLayout/README.md) | [card](../04-implementation/use-cases/chooseCourseLayout/README.md) | proposed |

### Use-case diagram

![Use cases](./use-case-overview.svg)

### By actor

#### Authenticated user

![Authenticated user](./use-cases-authenticated_user.svg)

#### Campus administrator

![Campus administrator](./use-cases-campus_administrator.svg)

#### Course creator

![Course creator](./use-cases-course_creator.svg)

#### Manager

![Manager](./use-cases-manager.svg)

#### System

![System](./use-cases-system.svg)

#### Teacher

![Teacher](./use-cases-teacher.svg)

#### Visitor

![Visitor](./use-cases-visitor.svg)

The index and its diagrams are generated from the use-case cards (front matter: `actor`, `package`, `includes`, `extends`). Edit the cards, not this table.

## Conventions

- `specification.puml` is the behavioural source of truth of a use case: a statechart with the Actor/System conversation.
- `wireframes.puml` is the interface prototype, even when it holds a single screen.
- Each use-case folder holds `README.md`, `specification.puml` and `wireframes.puml`.
