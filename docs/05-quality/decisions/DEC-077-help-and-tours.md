---
id: DEC-077
status: accepted
---

# DEC-077 Help topics declared by every screen, onboarding checklists and guided tours as data

- Date: 2026-11-18
- Context: Help that is not linked from the screen is not read; tours tied to fragile page structure break with every release.
- Options considered: a help link to the handbook home; hand-coded tours; declared help topics and tours as data on stable element ids (chosen).
- Decision:
  - Every route and contribution declares a help topic; a missing topic fails the documentation check.
  - The help panel shows the topic for the viewer's role and version, with the campus's terms and local pages.
  - Onboarding checklists per role tick themselves from the events of real actions.
  - Guided tours are steps on stable `data-help-id` elements; missing elements are skipped and reported.
- Consequences: Help is always one click away and matches the release; tours survive layout changes.
- Affected artifacts: `web/`, `handbook/`, every UI module.
