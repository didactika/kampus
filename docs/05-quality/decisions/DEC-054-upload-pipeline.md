---
id: DEC-054
status: accepted
---

# DEC-054 One upload pipeline with sniffed types, scanning and quotas

- Date: 2026-11-18
- Context: Several upload paths (web, API, plugins) with different checks let oversized or dangerous files in through the weakest one.
- Options considered: checks in each client; checks in each module; one pipeline every client goes through (chosen).
- Decision:
  - Every upload opens a session for a declared area, which sets the maximum size and the allowed types.
  - The media type is sniffed from the content; the declared type is only a hint.
  - The quota of the owning scope is checked when the session opens and updated in the attaching transaction.
  - A file is usable once its scan finishes clean; conversions run as jobs and keep the original.
- Consequences: The same limits for every client; quota errors are explicit and early.
- Affected artifacts: `crates/kampus-files`, `crates/kampus-web`.
