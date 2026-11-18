---
id: DEC-024
status: accepted
---

# DEC-024 Project Fluent messages, BCP 47 locales, one rich-text format

- Date: 2026-11-18
- Context: Every module shows text in many languages, with plurals and genders, and campuses rename terms (DEC-008, DEC-012). Concatenated strings and positional placeholders cannot be translated well.
- Options considered: gettext; ICU MessageFormat; Project Fluent (chosen).
- Decision:
  - User-facing strings are Fluent messages of their module, with named parameters, plural and gender rules, one file per locale (BCP 47).
  - The terminology layer renames terms inside messages per scope.
  - The locale comes from the user, then the campus, then the request; right-to-left is supported from the start.
  - User content is one sanitised rich-text format, sanitised on write and on render, or plain text.
  - Text is UTF-8 in NFC; natural keys people type compare case-insensitively.
- Consequences: Translators work on files with context; the handbook uses the same pipeline (DEC-011); no HTML is ever rendered unsanitised.
- Affected artifacts: Every module, `web/`, the handbook, the plugin SDK.
