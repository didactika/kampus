---
id: DEC-062
status: accepted
---

# DEC-062 ICU messages declared per module, language packs, customisations kept apart and CLDR formatting

- Date: 2026-11-18
- Context: Strings concatenated in code cannot be translated well; customisations stored over translations are lost on update; missing translations are found by users.
- Options considered: gettext catalogues; key and value strings per module; ICU MessageFormat messages declared per module with packs and scoped customisations (chosen).
- Decision:
  - Every user-facing string is a declared ICU message; a lint forbids literal strings in UI and errors.
  - Packs are versioned per module and language; customisations are separate rows per scope and survive updates.
  - The language is resolved from the course (forced), the person, the visitor's browser and the campus; fallbacks follow the language's parent chain and missing strings are counted.
  - Formatting uses CLDR for the resolved locale; right-to-left languages switch the client's direction.
- Consequences: Correct plurals and genders in every language; terminology (DEC-012) applies on top of every message.
- Affected artifacts: `crates/kampus-i18n`, the web client, every module's messages.
