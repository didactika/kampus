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
| Architecture | the dependency rules of the architecture (levels, ports), checked on the crate manifests |
| Schema | the schema linter of DEC-014 |
| Supply chain | `cargo deny` (licences compatible with the AGPL of DEC-085, advisories), `cargo audit`, npm audit |
| Documentation | link and diagram checks, schema dependencies and OpenAPI validation (`tools/docs-check`) |
| Repository | file headers (DEC-083), changelog entry and version consistency (DEC-081), crate dependency rules (`tools/repo-check`) |

## Deployment configuration

A file (`kampus.toml`) plus environment variables, which take precedence. Each role validates at start-up only the variables it uses and refuses to start when one is missing or invalid, listing every problem at once (DEC-020). The binary has three commands: `kampus-server serve` (the default), `kampus-server upgrade [--new-installation]` and `kampus-server check`, which exits with an error when the database does not match the build.

| Variable | Used by | Required | Description |
| --- | --- | --- | --- |
| `KAMPUS_ROLE` | all | no (default `all`) | `server`, `worker`, `scheduler` or `all` |
| `KAMPUS_PUBLIC_URL` | server, worker | yes | Public address of the campus, used in links and mail |
| `KAMPUS_HTTP_ADDRESS` | all | no (default `0.0.0.0:8080`) | Address of the HTTP listener: the API for `server`, health and readiness for every role |
| `KAMPUS_CONFIG_FILE` | all | no (default `kampus.toml` when present) | The configuration file; its keys are the variable names without `KAMPUS_`, in lower case |
| `KAMPUS_UPGRADE_ON_START` | all | no (default `false`) | Apply pending upgrade steps when the process starts (DEC-082); otherwise run `kampus-server upgrade` |
| `KAMPUS_DATABASE_URL` | all | yes | Connection to the database port (PostgreSQL first) |
| `KAMPUS_DATABASE_PREFIX` | all | no (default `kmp_`) | Installation prefix of every physical database name; set once at installation (DEC-039) |
| `KAMPUS_DATABASE_POOL_SIZE` | all | no | Connections per process |
| `KAMPUS_STORAGE_URL` | server, worker | yes | File storage: `file:///path` or `s3://bucket/prefix` |
| `KAMPUS_STORAGE_S3_*` | server, worker | with S3 | Endpoint, region and credentials of S3-compatible storage |
| `KAMPUS_CACHE_URL` | server, worker | no | Redis or Valkey (`redis://`, `rediss://`, `valkey://`); without it, in-process cache only (one instance) |
| `KAMPUS_SESSIONS_URL` | server | no | Sessions port; empty for the database, `redis://` for Redis or Valkey |
| `KAMPUS_QUEUE_URL` | all | no | Task queue port; empty for the database |
| `KAMPUS_SEARCH_URL` | server, worker | no | Search port; empty for PostgreSQL full-text search |
| `KAMPUS_SECRET_KEY` | all | yes | Key that encrypts stored secrets and signs cookies and cursors |
| `KAMPUS_MAIL_URL` | worker | yes | `smtp://` or a provider address |
| `KAMPUS_LOG_LEVEL` | all | no | Default `info` |
| `KAMPUS_LOG_FORMAT` | all | no (default `json`) | `json` lines for collectors, or `text` for development |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | all | no | OpenTelemetry collector (DEC-019) |
