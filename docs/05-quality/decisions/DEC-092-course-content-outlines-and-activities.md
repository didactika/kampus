---
id: DEC-092
status: accepted
---

# DEC-092 The content of a course is a required widget, presented by an outline plugin, with activities as plugins

- Date: 2026-11-23
- Context: Everything on the page of a course is a widget (DEC-090), but the content and the summary of a course are not like the others: a course without them is not a course. The owner wants them always present, at one size, movable but never removed or resized. The owner also wants how the content presents its sections and activities to be replaceable by a plugin, with today's presentation as the standard one, and activities kept outside the course module as plugins rendered inside the content. Plugins must not depend on each other: they reuse the core, so the platform stays clear, safe and efficient.
- Options considered:
  - The content and the summary as part of the page, outside the grid.
  - Ordinary widgets a teacher may remove.
  - Required widgets, with the presentation of the content delegated to a plugin of its own type (chosen).
- Decision:
  - **Required widgets.** A widget may declare that every page it goes on holds it (`required`): it is single, of one size, never removed, and it may move. The content and the summary of a course are required. The server refuses a layout without them and places them, in the first room that holds them, in a layout written before.
  - **Outlines.** How the content of a course presents its sections and activities is a plugin of the `outline` type, in `crates/outline/plugins/<name>`: the standard outline (`sections`) is today's presentation; others, such as by weeks or on a single page, may follow. A course chooses its outline (a setting of the course, with a default by scope); the content widget renders the outline of its course, and an outline may let the teachers arrange what is inside it.
  - **Activities.** Each kind of activity is a plugin of the `activity` type, in `crates/activity/plugins/<name>`, on the contract `activities` defines today (RN-075), which moves to `kampus-activity` with the first plugin. An outline shows activities through the registry of `activities`, never by depending on an activity plugin.
  - **Dependencies.** A plugin depends on core modules and on the crate of its type only; what one type needs of another goes through the core that owns its registry. The core keeps the services, permissions, cache and events every plugin reuses.
  - **Locks by scope:** a platform, a tenant, a campus or a category may lock a widget for the scopes below it: every layout below shows it as placed there, pinned, and ignores what a writer sends of it; a course, a section and a person lock nothing, since nothing below them arranges the same page. It applies to the page of a course and to the home of each person alike.
- Consequences:
  - The content and the summary of a course are required widgets from this release; the summary keeps one size.
  - `courses` keeps the data of sections and the setting of the outline of a course; `kampus-outline` and `kampus-activity` arrive as type crates with the standard outline and the first activities, each in its own pull request.
  - The editor of a page shows required widgets without a way to remove them, with the lock of their fixed size.
- Affected artifacts: the [plugins guide](../../03-design/plugins/README.md), [widget plugins](../../03-design/plugins/widget.md), `kampus-module`, `kampus-widget`, `customise`, `courses`, `activities`, the web client.
