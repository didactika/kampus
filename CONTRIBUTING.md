# Contributing to Kampus

## Branches

- `main` holds released versions only. Nobody pushes to it; a release branch is merged into it when a version ships.
- `release/<version>` (now `release/0.1.0`) integrates the work of the next version.
- `support/X.Y` receives the patch releases of a supported minor version (DEC-081).
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

Everything runs in containers: nothing but Git and Docker is needed on the host. The `rust` service of `compose.yaml` is a toolbox with the pinned toolchain (`rust-toolchain.toml`), `cargo deny` and `cargo audit`, next to PostgreSQL 17.

```bash
docker compose up -d postgres                     # PostgreSQL 17 on localhost:55432 (kampus / kampus)
docker compose run --rm rust cargo fmt --all
docker compose run --rm rust cargo clippy --workspace --all-targets --locked -- -D warnings
docker compose run --rm rust cargo test --workspace --locked
docker compose run --rm rust cargo deny --all-features check
docker compose run --rm rust cargo audit
docker compose -f tools/docs-check/compose.yaml run --rm repo
```

The same gates run in CI on every pull request (`.github/workflows/ci.yml`).

## Source files

- Every file outside `docs/` starts with the standard header of DEC-083, in the comment syntax of its type: the KAMPUS banner, the notice of the GNU Affero General Public License (DEC-085), and `@package`, `@copyright`, `@author` (name and address), `@license` and `@since`. `docker compose -f tools/docs-check/compose.yaml run --rm repo --fix --author "Your Name <you@example.org>"` adds missing headers and rewrites outdated ones, keeping their `@since`. In Rust:

  ```rust
  // ------------------------------------------------------------------------------
  //
  //    *   *    *    *   *  ****   *   *   ****
  //    *  *    * *   ** **  *   *  *   *  *
  //    ***    *****  * * *  ****   *   *   ***
  //    *  *   *   *  *   *  *      *   *      *
  //    *   *  *   *  *   *  *       ***   ****
  //
  //    Knowledge Academic Modular Platform for Universal Study
  //
  // ------------------------------------------------------------------------------
  //
  // This file is part of Kampus - https://github.com/didactika/kampus
  //
  // Kampus is free software: you can redistribute it and/or modify it under the terms of the
  // GNU Affero General Public License as published by the Free Software Foundation, either
  // version 3 of the License, or (at your option) any later version.
  //
  // Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
  // even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
  // GNU Affero General Public License for more details.
  //
  // You should have received a copy of the GNU Affero General Public License along with Kampus.
  // If not, see <https://www.gnu.org/licenses/>.
  //
  // @package   kampus-courses
  // @copyright 2026 Didactika
  // @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
  // @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
  // @since     0.1.0
  //
  // SPDX-License-Identifier: AGPL-3.0-or-later
  // ------------------------------------------------------------------------------
  ```
- Every item has a documentation comment above it, public or private (`//!` and `///` in Rust, TSDoc in TypeScript and Svelte): what it is for, not how it works. Comments inside bodies only for what the code cannot say.

## Versions, changelog and releases

- Kampus has one semantic version, in `[workspace.package]` of `Cargo.toml` (DEC-081). On `release/X.Y.Z` it equals `X.Y.Z`.
- Every pull request that changes anything outside `docs/` adds its entry under `## [Unreleased]` in `CHANGELOG.md`, in one of the groups Added, Changed, Deprecated, Removed, Fixed, Security or **Upgrade** (every action an operator must take).
- Upgrades must work from any supported release straight to the new one (DEC-082): migrations are forward-only and never edited once released, interfaces change additively within a major version, and whatever is removed is deprecated first.
- Releasing `X.Y.Z`: move the `Unreleased` entries into `## [X.Y.Z] - <date>`, merge `release/X.Y.Z` into `main`, tag `vX.Y.Z`, cut `support/X.Y` for its patch releases and open `release/<next>`.
