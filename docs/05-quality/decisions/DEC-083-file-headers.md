---
id: DEC-083
status: accepted
---

# DEC-083 A standard header in every file and documentation comments above every item

- Date: 2026-11-18
- Context: Files moved or copied out of context lose their origin, ownership and licence; code without documentation above its items is hard to maintain, while narrative comments inside bodies go stale.
- Options considered: no headers; a licence line only; a full standard header plus documentation comments on items, checked in CI (chosen).
- Decision:
  - Every source and configuration file outside `docs/` starts with the Kampus header, in the comment syntax of its type: the project line, the copyright of Didactika, the licence reference and its SPDX identifier, then `@package` (the crate or component), `@author` (who created the file) and `@since` (the version that added it).
  - Files that cannot hold comments (JSON, lock files, images) and the specification in `docs/` are exempt.
  - Every item gets a documentation comment above it, public or not (Rust: `//!` for modules, `///` for items; TypeScript and Svelte: TSDoc), saying what it is for, not how it works. Comments inside bodies are kept for what the code cannot say.
  - `tools/repo-check` fails when a file lacks its header and adds missing headers with `--fix`; the Rust lints `missing_docs` and `missing_docs_in_private_items` fail on undocumented items.
- Consequences: Every file says what it belongs to and under which terms; the generated references (DEC-078) have text for every item.
- Affected artifacts: every file, `tools/repo-check`, the workspace lints.
