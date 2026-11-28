---
id: ADMINISTRATION
status: accepted
origin: requirements
sourceOfTruth: model
---

# The administration

How Kampus is administered from the client: where the administration is, how it is organised, and the few kinds of page every part of it is built from, so a new area or a module adds pages without changing the frame or inventing a layout.

## Where it is

The administration lives at `/admin`. The main bar of the campus shows **Administration** to whoever may administer anything in it: change a setting of the campus, or read its accounts. It is never in the menu of the account, which holds what belongs to the person (their account, their language, their preferences).

## How it is organised

A side panel holds a search and the map of the administration; the page open is beside it. On a narrow screen the panel opens from a button above the page.

The map is a list of **areas**, each with its **pages**. An area shows only the pages the person may open, and an area without any is not shown.

| Area | Pages |
| --- | --- |
| Overview | The areas, each with its pages and what needs attention |
| People | Accounts (list, invitation, account page), system groups, identity providers, service accounts |
| Courses | Categories and courses (the tree, a course page), course requests |
| Permissions | Roles (a list, and a page per role with its permissions by module), assignments, check access |
| Settings | One page per module that declares settings |
| Appearance | Theme, menus, terminology, features, custom fields, the home of the campus |
| Communication | Notification channels and templates |
| Privacy | Policies, retention, requests |
| Automation | Rules, packages |

The search finds pages by their name and settings by their name, their help and their key, and leads to the setting itself on its page.

## The kinds of page

Every page of the administration is one of four kinds, each with one layout.

| Kind | What it holds |
| --- | --- |
| List | The head of the page with its main action (Invite, Create); a toolbar with the search and the filters that matter for that list; a table with a column of selection, the columns that tell items apart and one action menu per row; a bar of bulk actions while rows are selected; the pager. A row opens its item. |
| Item | The head with the name, its status and its main actions; tabs for what the item has (profile, roles, enrolments, history); each tab a form or a list. |
| Form | Sections of fields; changes wait in the floating bar until they are saved; leaving with changes asks first. Creating opens a dialog with the few fields needed, and the item page holds the rest. |
| Settings | The settings a module declares, in the sections of their groups. A setting shows its name, what it does and its field; it shows more only when there is something to tell: that it is not saved, inherited, locked, needs a restart, or differs from its default (then the default is said). Its actions (lock for the levels below, take the inherited value again, history) are in its menu. The level (this campus or the whole platform) is chosen at the head of the page when the person may set both. |

Pages say little: a short lead under the title, help where a field needs it, and nothing that repeats what the page already shows.

## How a module adds pages

Settings need no page of their own: a module declares them with a group, and the settings page of the module shows them. Any other page of a module is a page of its UI module, registered with the area it belongs to, as widgets are registered with their type; the map takes it from the registry, so adding a module adds its pages and the frame does not change.
