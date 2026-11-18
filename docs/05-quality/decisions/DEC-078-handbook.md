---
id: DEC-078
status: accepted
---

# DEC-078 Handbook structure: guides per audience, page types and generated references

- Date: 2026-11-18
- Context: DEC-011 requires documentation for every role with each release. Hand-written references fall behind the code; guides organised by module instead of by task are hard to use.
- Options considered: one manual per module; a wiki; guides per audience with typed pages and generated references (chosen).
- Decision:
  - One guide per audience; pages are tasks, concepts, references, tutorials or release notes.
  - Task outlines come from the use cases; references (permissions, settings, events, API, CLI, configuration, errors, terms) are generated from declarations.
  - Screenshots come from the end-to-end tests; tutorials are tested in CI.
  - Release notes come from the highlights register.
- Consequences: The handbook matches the release it ships with (RNF-020) and costs little to keep current.
- Affected artifacts: `didactika/kampus-docs` (DEC-084), the documentation checks in CI.
