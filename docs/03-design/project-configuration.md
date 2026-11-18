---
id: CFG
status: accepted
origin: requirements
sourceOfTruth: model
---

# Project configuration

| Item | Value |
| --- | --- |
| Language | Rust, edition 2024, stable toolchain pinned in `rust-toolchain.toml` (DEC-001) |
| Framework | Axum on Tokio, tower middleware |
| Persistence | SQLx over the database port, PostgreSQL 17 first (DEC-009, DEC-014) |
| Web client | SvelteKit with Svelte 5 and TypeScript, Node 24 (DEC-004) |
| Messages | Project Fluent (DEC-024) |
| API description | OpenAPI 3.1, generated from the code (DEC-025) |
| Testing | `cargo test` with tests against PostgreSQL in Docker; Playwright for the web client and end-to-end tests |
| Build and run | Docker only: one image with three roles (DEC-020), Docker Compose for development |

## Quality gates (every pull request)

| Gate | Tool |
| --- | --- |
| Formatting | `cargo fmt --check`, Prettier for the web client |
| Lints | `cargo clippy` with warnings as errors, ESLint |
| Tests | `cargo test` against PostgreSQL, Playwright end to end |
| Architecture | the dependency rules of the architecture (levels, ports) |
| Schema | the schema linter of DEC-014 |
| Supply chain | `cargo deny` (licences of DEC-010, advisories), `cargo audit`, npm audit |
| Documentation | link and diagram checks (`tools/docs-check`) |

## Deployment configuration

A file (`kampus.toml`) plus environment variables, which take precedence. Each role validates at start-up only the variables it uses and refuses to start when one is missing or invalid (DEC-020).

| Variable | Used by | Required | Description |
| --- | --- | --- | --- |
| `KAMPUS_ROLE` | all | no (default `all`) | `server`, `worker`, `scheduler` or `all` |
| `KAMPUS_PUBLIC_URL` | server, worker | yes | Public address of the campus, used in links and mail |
| `KAMPUS_DATABASE_URL` | all | yes | Connection to the database port (PostgreSQL first) |
| `KAMPUS_DATABASE_POOL_SIZE` | all | no | Connections per process |
| `KAMPUS_STORAGE_URL` | server, worker | yes | File storage: `file:///path` or `s3://bucket/prefix` |
| `KAMPUS_STORAGE_S3_*` | server, worker | with S3 | Endpoint, region and credentials of S3-compatible storage |
| `KAMPUS_CACHE_URL` | server, worker | no | Redis or Valkey; without it, in-process cache only (one instance) |
| `KAMPUS_SECRET_KEY` | all | yes | Key that encrypts stored secrets and signs cookies and cursors |
| `KAMPUS_MAIL_URL` | worker | yes | `smtp://` or a provider address |
| `KAMPUS_LOG_LEVEL` | all | no | Default `info` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | all | no | OpenTelemetry collector (DEC-019) |
