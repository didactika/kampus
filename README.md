<p align="center"><img src="docs/assets/kampus-icon.svg" alt="Kampus" width="128"></p>

# Kampus

**KAMPUS** — **K**nowledge **A**cademic **M**odular **P**latform for **U**niversal **S**tudy.

A modular learning platform written from scratch in Rust: courses, enrolment, roles and permissions with a section scope, activities with exceptions for any learner or group, academic records, progress, communication and administration. Everything is customisable per campus, category, course or section, plugins run sandboxed as WebAssembly components, and migration tools bring existing sites and plugins over.

## Status

Version 0.1.0 is in progress on the `release/0.1.0` branch: the specification of the platform is complete and the core is being built. Changes are recorded in the [changelog](CHANGELOG.md). See the [documentation](docs/README.md) and the [milestones](../../milestones).

## Repository

| Path | Content |
| --- | --- |
| [`docs/`](docs/README.md) | The specification: requirements, domain model, design, decisions and highlights |
| [`tools/docs-check/`](tools/docs-check/) | Documentation checks run in CI and locally with Docker |
| `crates/` | Rust workspace, one crate per module |
| [`CHANGELOG.md`](CHANGELOG.md) | Changes of every version and the upgrade steps |
| [`tools/repo-check/`](tools/repo-check/) | File headers, version and changelog, and the dependency rules of the crates |
| [`tools/dev/`](tools/dev/) | Toolbox image for local development and CI parity |
| `web/` | Web client (coming) |
| [`kampus-docs`](https://github.com/didactika/kampus-docs) | Documentation for every role, in its own repository (DEC-084) |

## Stack

Rust (Axum, SQLx, Tokio); pluggable database, file storage and cache, with PostgreSQL, local and S3 storage, and Redis first; plugins as WebAssembly components; a SvelteKit web client; a REST API described with OpenAPI, plus webhooks. Decisions are recorded in [`docs/05-quality/decisions/`](docs/05-quality/decisions/).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

Kampus is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License, version 3 or (at your option) any later version (DEC-085). Copyright (c) 2026 Didactika. See [LICENSE](LICENSE).
