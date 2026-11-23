---
id: DESIGN-PLUGINS-WIDGET
status: accepted
---

# Widget plugins

A widget shows something on a page arranged as a grid: the home of each person, the page of a course (DEC-090). Every widget, of the core or not, is a plugin of this type (DEC-091), in `crates/widget/plugins/<name>`, built on `kampus-widget`. The [plugins guide](./README.md) has the rules every plugin keeps; this page adds what a widget declares and how it renders.

## Structure

```text
crates/widget/plugins/<name>/
├── Cargo.toml                       kampus-widget-<name>, kind module; depends on kampus-common, kampus-db,
│                                    kampus-module and kampus-widget
├── version.toml                     [requires]: customise, and the core modules whose data it shows
├── src/
│   ├── lib.rs                       pub mod db, module, permissions, settings
│   ├── module.rs                    MODULE, named widget_<name>, with widgets: settings::customise::WIDGETS
│   ├── db/steps.rs                  usually no steps: what it shows belongs to core modules
│   ├── permissions/                 usually none: it shows only what the reader may already see
│   └── settings/customise.rs        WIDGETS: its one declaration
├── tests/unit/conformance.rs        kampus_widget::conformance::problems(&MODULE) is empty
└── web/
    ├── package.json                 @kampus/widget-<name>, exports ./src/index.ts
    ├── src/index.ts                 export const widget: WidgetPlugin = { code: "widget_<name>", render }
    ├── src/<Name>Widget.tsx         the server component that renders it
    └── strings/en.json, es.json     widget_<name>.name, widget_<name>.about, and its own texts
```

## The declaration

| Field | Meaning |
| --- | --- |
| `code` | The name of the plugin, `widget_<name>` |
| `summary` | What it shows, in English, for the administration |
| `icon` | Its icon, by the name the design system gives it |
| `single` | A page holds it once at most (the welcome, the content of a course) |
| `required` | Every page it goes on holds it, at one size and never removed, though it may move (the content and the summary of a course); a required widget is single and of one size |
| `sizes` | For each page kind it goes on, its smallest, default and largest size in cells and how it may be resized; it goes on no other page kind. `WidgetSize::fixed` gives one size |
| `bands` (of a size) | The heights it takes between two widths, when it needs fewer rows wider: the welcome takes up to six rows at six columns and up to four when wider. Every width is in one band at most, and the default size keeps to its band |
| `settings` | Its settings, each with its kind and default, whether it may not be left empty (`required`) and whether a text takes one value per language (`translatable`). The kind `rich_text` holds HTML in the format of user content, sanitised by the server when it is written and when it is read |

The server checks the sizes against the grid of each page kind when it reads the declarations, refuses a layout with a rectangle outside them, two of a single widget or a required one left out, and places a required widget that a layout written before lacks in the first room that holds it.

## Rendering

- The render function is a server component. It receives its settings with their defaults, the page it is on (the campus and, on a course, what the page read of it) and its **size in cells**.
- It must look right at **every size it allows**, from its smallest to its largest: it chooses how much to show from its size (`fitting` and `heightOf` of the host API help), keeps its heading and footer fixed and scrolls only its list, and never spills out of its rectangle.
- It reads data only through the host API and the API of the core, as the reader; it never calls another plugin.
- Its texts come from its strings; it writes none in code.
- A text the person writes on the widget is rendered with `EditableText` or `EditableRichText` of the host API: outside the editor they show the text in the language of the reader, and while the page is arranged they are its fields. Rich text read elsewhere is shown with `RichText`, and a plugin adds formats to the rich editor with `registerEditorExtension`.

## Checklist

1. The declaration passes the conformance rules, and `tests/unit/conformance.rs` runs them.
2. The widget is correct at its smallest and its largest size on every page kind it declares, in light and dark, and on a narrow screen.
3. `single` and `required` say what the widget is, not what a page happens to hold.
4. Its strings exist in every language of the repository.
5. `tools/repo-check` is green: folder, package, imports.
