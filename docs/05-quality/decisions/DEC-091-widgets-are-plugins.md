---
id: DEC-091
status: accepted
---

# DEC-091 Widgets are plugins of the widget type, and every plugin type has one folder and one structure

- Date: 2026-11-23
- Context: The widgets of the core lived as loose files of the web client and as declarations scattered in the modules whose data they show. The owner wants them grouped as plugins of the widget type, every one following one structure, so a core widget can be removed as easily as an external one and new ones are written the same way. The owner also wants every kind of plugin, and the folder each lives in, written down now, so the platform grows without breaking its rules. The structure of native plugins was foreseen (DEC-087: a plugin of a type a module defines lives in `crates/<owner>/<type>/<name>`), but no plugin existed yet.
- Options considered:
  - Keep widgets in the modules whose data they show, with their screens in the web client.
  - One crate per widget plugin inside the module that defines the type.
  - A crate for the type, with its plugins inside it, each with its web part as a package of its own (chosen, after the owner preferred types outside the modules).
  - One crate holding every core widget.
- Decision:
  - **Plugin types and their folders.** Every plugin type has a crate of its own, of kind `type`: `crates/<type>`, with the vocabulary of its declarations (`declaration/`) and the rules its plugins keep (`conformance/`). Its plugins live in `crates/<type>/plugins/<name>`, the same folder name in every type, package `kampus-<type>-<name>`, component `<type>_<name>`; each runs the conformance rules of its type in its own tests. A plugin depends on core modules and on the crate of its type only, never on another plugin, and only the server depends on plugins. The [plugins guide](../../03-design/plugins/README.md) lists every type, its owner, its folder and its contract, and has one page per type with its structure and a checklist. A plugin from outside the repository has the same structure and is declared in `crates/server/modules.toml` with its git or registry source.
  - **A widget is a plugin of the `widget` type,** built on `kampus-widget`, in `crates/widget/plugins/<name>`. It declares exactly one widget, whose code is the name of the plugin (`widget_catalogue`), with its icon, its sizes on each page kind (DEC-090), whether a page holds it once at most, whether every page holds it (DEC-092), and its settings. No other component declares widgets, and the checks of the repository and of the server refuse one that does.
  - **The web part of a plugin is a package of the workspace,** in the `web/` folder of its crate (`@kampus/widget-catalogue`), with its strings in `web/strings/<language>.json` under the prefix of the plugin. It imports only the design system (`@kampus/ui`), the types of the API (`@kampus/sdk`) and the host API of the web client: `@kampus/web/host` on the server and `@kampus/web/host/client` in the browser. It never imports the inside of the client. The registry of the client is written from the plugin folders, so a plugin added or removed is a folder added or removed.
  - **Single widgets.** A widget may declare that a page holds it once at most (the welcome, the content of a course); the server refuses a layout with two, and the editor does not offer it again.
  - **Settings of components** (to come with the settings module): the administration lists the platform, every module and plugin with its version, and offers installing a plugin from a git repository and going back to a version the code registers when the server may write its files.
- Consequences:
  - The widgets of the core move into `crates/widget/plugins/` and their codes change to the names of their plugins; an upgrade step of `customise` renames the placements of existing layouts.
  - Removing a widget plugin is removing its folder: the registries are written again, and the layouts that held it skip it (RN-216).
  - `tools/repo-check` places the plugin crates in the server's registry after the core modules, checks the layout of type and plugin crates, refuses a plugin that depends on anything but the core and its type, and checks the imports of their web part.
  - The types whose contracts live in a module today (activities, enrolment methods) move to a crate of their own with their first plugin.
- Affected artifacts: the [plugins guide](../../03-design/plugins/README.md), the [module layout](../../03-design/module-layout.md), the [plugin SDK](../../03-design/sdk/README.md), `kampus-module` (widget declarations), `customise`, the modules that declared widgets, the web client, `tools/repo-check`.
