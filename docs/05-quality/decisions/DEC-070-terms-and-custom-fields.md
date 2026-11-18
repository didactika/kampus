---
id: DEC-070
status: accepted
---

# DEC-070 Terms with grammatical forms and typed custom fields owned through link tables

- Date: 2026-11-18
- Context: Renaming words by replacing strings breaks grammar (plurals, gender, articles); custom fields stored as text for every type cannot be validated, sorted or reported on.
- Options considered: string replacement in language packs; custom fields as key and value text; declared terms with forms and typed fields (chosen).
- Decision:
  - Modules declare terms with grammatical forms per language; messages refer to terms, so one rename is right everywhere.
  - Custom fields are defined per entity kind and scope with a type, validation, visibility and permissions.
  - Values are typed columns checked against the field's type, owned through each module's link table like files.
  - Choices are rows with stable codes.
- Consequences: Campuses use their own words in every language; custom data can be filtered, reported and exported reliably.
- Affected artifacts: `crates/customise`, `crates/i18n`, every module that accepts custom fields.
