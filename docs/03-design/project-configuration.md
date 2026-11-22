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

The binary has its commands (`serve`, the default, `install`, `upgrade`, `check`, `components`, `history`, `downgrade`, `uninstall`) and one configuration for every role.

### Installing

`kampus-server install` sets an installation up in one go:

1. It reads `config.toml` (or the file `KAMPUS_CONFIG_FILE` names) and the environment, and asks only for what is still missing: the site's address, the database, the schema prefix, the file storage, the cache and how campuses are served. Without a terminal, or with `--non-interactive`, it asks nothing and lists every missing value at once.
2. It generates the secret key when there is none, and writes the file, readable only by its owner. Values given by variables stay in the variables.
3. When the database already holds the installation, it changes nothing in it and says whether `kampus-server upgrade` is due: a filled `config.toml` pointing at an installed database is all a new server needs.
4. Otherwise it creates the installation, the first campus with its roles, and the campus administrator: email, username and password are asked, or read from `KAMPUS_ADMIN_EMAIL`, `KAMPUS_ADMIN_USERNAME` and `KAMPUS_ADMIN_PASSWORD`, which only the installer reads.

```sh
docker compose run --rm -it server install
```

### The configuration file

`config.toml` has one table per section; [`config.example.toml`](https://github.com/didactika/kampus/blob/release/0.1.0/config.example.toml) at the root of the repository shows them all. Every key has its environment variable, `KAMPUS_<SECTION>_<KEY>`, which takes precedence over the file. Each role checks at start-up only what it reads and refuses to start when a value is missing or invalid, listing every problem at once. The file holds secrets: keep it out of version control and readable only by Kampus.

| Key | Variable | Used by | Default | Description |
| --- | --- | --- | --- | --- |
| `site.url` | `KAMPUS_SITE_URL` | server | required | The public address of the site, used in links and mail |
| `server.role` | `KAMPUS_SERVER_ROLE` | all | `all` | `server`, `worker`, `scheduler` or `all` |
| `server.http_address` | `KAMPUS_SERVER_HTTP_ADDRESS` | all | `0.0.0.0:8080` | The address of the HTTP listener |
| `server.upgrade_on_start` | `KAMPUS_SERVER_UPGRADE_ON_START` | all | `false` | Apply pending upgrades when the process starts |
| `database.url` | `KAMPUS_DATABASE_URL` | all | required | The PostgreSQL connection |
| `database.prefix` | `KAMPUS_DATABASE_PREFIX` | all | `kmp_` | The prefix of every schema, set once at installation |
| `database.pool_size` | `KAMPUS_DATABASE_POOL_SIZE` | all | `10` | Connections per process |
| `storage.url` | `KAMPUS_STORAGE_URL` | server, worker | required | `file:///path` or `s3://bucket/prefix` |
| `storage.s3_endpoint`, `s3_region`, `s3_access_key_id`, `s3_secret_access_key` | `KAMPUS_STORAGE_S3_…` | server, worker | with `s3://` | The S3-compatible store: endpoint (omit for AWS), region and credentials |
| `cache.url` | `KAMPUS_CACHE_URL` | server, worker | in-process | `redis://`, `rediss://` or `valkey://`; without it only one instance may run |
| `scan.url` | `KAMPUS_SCAN_URL` | server, worker | none | `clamd://host:port` (port 3310 when absent); every upload is scanned and infected content refused; without it uploads are kept unscanned |
| `mail.url` | `KAMPUS_MAIL_URL` | worker | none | `smtp://host:port` in the clear, `smtp://host:port?tls=required` with STARTTLS, `smtps://host:port` over TLS; without it the email channel writes to the log what it would send |
| `mail.from`, `username`, `password` | `KAMPUS_MAIL_…` | worker | `from` with `mail.url` | The sender of every message (`Kampus <no-reply@example.org>`), and the account to sign in to the server with, both or neither |
| `push.url` | `KAMPUS_PUSH_URL` | worker | none | `webpush://` sends push notifications to the push services of browsers; without it the push channel writes to the log what it would send |
| `push.vapid_private_key`, `subject` | `KAMPUS_PUSH_…` | worker | with `push.url` | The VAPID key of the installation, from `kampus-server push-key`, kept while devices are subscribed with it; and who push services contact, `mailto:` or `https:` |
| `security.secret_key` | `KAMPUS_SECURITY_SECRET_KEY` | server, worker | required | 32 bytes in base64 that seal stored secrets; generated by the installer, never changed afterwards |
| `tenancy.mode` | `KAMPUS_TENANCY_MODE` | all | `single` | `single`: one campus at the site's address; `multi`: a campus per domain |
| `tenancy.base_domain` | `KAMPUS_TENANCY_BASE_DOMAIN` | all | with `multi` | The domain under which each campus has its own (`a.campuses.example.org`) |
| `log.level` | `KAMPUS_LOG_LEVEL` | all | `info` | `error`, `warn`, `info`, `debug` or `trace` |
| `log.format` | `KAMPUS_LOG_FORMAT` | all | `json` | `json` lines for collectors, or `text` |

`KAMPUS_CONFIG_FILE` names another file; the container image reads `/etc/kampus/config.toml`. Mail, sessions, the task queue and search get their sections with the modules that use them.
