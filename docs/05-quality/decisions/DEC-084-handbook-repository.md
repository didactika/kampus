---
id: DEC-084
status: accepted
---

# DEC-084 The handbook in its own repository, versioned with Kampus and previewed locally at every iteration

- Date: 2026-11-18
- Context: DEC-011 put the handbook in `handbook/` of this repository. The owner wants the product documentation apart from the code and the specification, reviewable on localhost at every iteration, with the look of Didactika's documentation portal.
- Options considered: `handbook/` in this repository; a separate private repository versioned with Kampus (chosen).
- Decision:
  - The handbook lives in the private repository `didactika/kampus-docs`: a Starlight site with Didactika's documentation palette, typography and a Kampus Docs logo, in English and Spanish, with one guide per audience (learners, teachers, administrators, operators, integrators, plugin developers, platform developers).
  - It is versioned with Kampus: its `release/X.Y.Z` branch documents Kampus `X.Y.Z`, `main` holds released versions, and it keeps its own changelog.
  - It reads this repository (use cases and their status from the traceability table, the changelog, and later the generated references of DEC-078) instead of copying it; `docker compose up -d` serves it at http://localhost:8096 and refreshes it while both repositories change.
  - Every pull request of Kampus that changes what a role sees or does updates the matching guide in `kampus-docs` in the same iteration.
- Consequences: The code repository holds code and specification only; the documentation is always reviewable at the state of the work.
- Affected artifacts: `didactika/kampus-docs`, DEC-011, DEC-078, the help topics of DEC-077.
