# Contributing to Kampus

## Branches

- `main` holds released versions only. Nobody pushes to it; a release branch is merged into it when a version ships.
- `release/<version>` (now `release/0.1.0`) integrates the work of the next version.
- Work happens on short branches cut from the release branch: `design/<module>`, `feat/<module>`, `fix/<topic>`, `docs/<topic>`, `chore/<topic>`. Each one is merged into the release branch through a pull request.

## Commits and pull requests

- Conventional Commits in English: `<type>(<scope>): <summary>`, imperative, lower case. Scopes: `docs`, the crate or module name, `web`, `converter`, `ci`, `repo`.
- One pull request per issue (or a small group of related issues), with `Closes #n`, targeting the release branch.
- A pull request is merged when CI is green.

## Issues and milestones

- One milestone per version (`0.1.0`, …).
- One issue per task, labelled with its area (`area:permissions`, `area:courses`, …) and its kind (`kind:design`, `kind:code`, `kind:docs`, `kind:chore`).

## Documentation

- Code, identifiers, documentation, commits and pull requests are in English, using the Kampus glossary (DEC-012).
- Every architectural or product decision is a record in `docs/05-quality/decisions/`.
- Every design choice that solves a known limitation gets an entry in `docs/05-quality/highlights.md`, in the same pull request.
- Diagrams are PlantUML sources with their rendered SVG next to them.
- Check the documentation locally before opening a pull request (Docker is the only requirement):

  ```bash
  docker compose -f tools/docs-check/compose.yaml run --rm check
  docker compose -f tools/docs-check/compose.yaml run --rm openapi
  docker compose -f tools/docs-check/compose.yaml down
  ```

## Toolchain

Everything runs in containers: nothing but Git and Docker is needed on the host.
