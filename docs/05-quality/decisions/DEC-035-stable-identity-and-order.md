---
id: DEC-035
status: accepted
---

# DEC-035 Stable identifiers and fractional sort keys for everything that is ordered

- Date: 2026-11-18
- Context: Sections, course modules, categories and courses are reordered all the time. When their position is their identity, every move renumbers siblings and breaks links, highlights and anything that stored a number.
- Options considered: dense positions renumbered on move; linked lists; fractional sort keys with stable ids (chosen).
- Decision:
  - Everything ordered is referenced by its UUID, never by its position.
  - Order within a parent is a fractional sort key: inserting or moving writes only the moved row; a rare job rebalances the keys of one parent.
  - Order is local to each parent; there is no global order and no limit per parent.
- Consequences: Moves are cheap and safe under concurrency; links and highlights survive any reorganisation; the API addresses everything by id.
- Affected artifacts: `crates/kampus-courses`, `crates/kampus-activities`, the course editor.
