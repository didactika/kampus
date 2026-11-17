---
id: DEC-006
status: accepted
---

# DEC-006 moodle2kampus in three layers

- Date: 2026-11-17
- Context: Moodle plugins are PHP; a fully automatic translation to Kampus plugins is not reliable.
- Options considered: run PHP plugins in a compatibility runtime; fully automatic translation; layered conversion.
- Decision: A deterministic translator for the declarative parts (version, schema, capabilities, services, strings, templates, events, tasks, settings); assisted conversion of the logic (source analysis → design model → Kampus plugin SDK) with generated tests; a report of what needs human review. It relies on an API mapping table maintained with the converter.
- Consequences: Predictable results for the declarative part; reviewed results for logic; converted plugins keep the licence of the original plugin.
- Affected artifacts: `tools/moodle2kampus/`, the API mapping table, the plugin SDK.
