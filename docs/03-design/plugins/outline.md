---
id: DESIGN-PLUGINS-OUTLINE
status: accepted
---

# Outline plugins

An outline is how the content of a course presents its sections and activities: in their order, by weeks, on one page… Every outline is a plugin of this type (DEC-092), in `crates/outline/plugins/<name>`, built on `kampus-outline`. The [plugins guide](./README.md) has the rules every plugin keeps.

## How a course gets its outline

- A course chooses its outline with the setting `courses.outline`, which a platform, a campus or a category sets for the courses below it and a course sets for itself. The standard outline, `outline_sections`, is the default.
- The read of a course gives its outline already resolved. When the outline it names is no longer installed, the course is shown with the standard one, so removing an outline plugin never breaks a course.
- The page of a course finds the outline in the registry of the client and gives it to the content widget in its context; no widget depends on an outline plugin.

## Structure

```text
crates/outline/plugins/<name>/
├── Cargo.toml                       kampus-outline-<name>, kind module; depends on kampus-common, kampus-db,
│                                    kampus-module and kampus-outline
├── version.toml                     [requires]: courses, and the core modules whose data it shows
├── src/settings/courses.rs          OUTLINES: its one declaration (code, summary, icon)
├── tests/unit/conformance.rs        kampus_outline::conformance::problems(&MODULE) is empty
└── web/
    ├── package.json                 @kampus/outline-<name>, exports ./src/index.ts
    ├── src/index.ts                 export const outline: OutlinePlugin = { code: "outline_<name>", render }
    └── strings/en.json, es.json     outline_<name>.name, outline_<name>.about
```

## Rendering

The render function is a server component. It receives the course as its page read it (sections, course modules, progress, grades, dates), whether its sections start folded, and the language of the reader. It shows activities through the host API, never by depending on an activity plugin.

## Checklist

1. The declaration passes the conformance rules, and `tests/unit/conformance.rs` runs them.
2. A course with no sections, with many, and with hidden ones reads well in light and dark and on a narrow screen.
3. Its strings exist in every language of the repository.
4. `tools/repo-check` is green: folder, package, imports.
