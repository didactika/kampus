---
id: DEC-009
status: accepted
---

# DEC-009 Pluggable infrastructure: databases, file storage, cache and more

- Date: 2026-11-17
- Context: The owner requires Kampus to run on different relational databases, file storage systems and cache systems. The first release ships one stack and support widens over time. Common weak points of such abstractions must be avoided: dialect gaps found only at run time, a single local file directory by default, and cache invalidation that is manual and per node.
- Options considered:
  - One hand-written store per database for every module: the most control, but N modules × M databases.
  - A full ORM for all databases: less code, weaker control and performance.
  - Ports per concern, with one implementation per family plus a conformance suite (chosen).
- Decision: every piece of infrastructure is a **port** (a Rust trait in the core) with interchangeable **adapters**, chosen by configuration at start-up.

  | Port | First release | Next adapters | Why |
  | --- | --- | --- | --- |
  | Database | PostgreSQL | MySQL and MariaDB, SQLite (development and small installations), then Microsoft SQL Server and Oracle | Schema written once in a neutral format and turned into migrations per dialect; queries through a dialect-neutral builder over the SQLx drivers; **every module test runs against every supported database in CI**, so gaps show up before a release, not in production |
  | File storage | Local disk and S3-compatible (AWS S3, MinIO, Cloudflare R2, and others) | Azure Blob, Google Cloud Storage, WebDAV, SFTP, through one storage layer (Apache OpenDAL) | Content-addressed storage with deduplication, streaming uploads and downloads, signed URLs, optional encryption at rest, virus scanning hook, moving files between backends without downtime |
  | Cache | In-process (moka) and Redis or Valkey | Memcached | Two levels (local in front of shared), typed cache definitions declared by each module, invalidation through the event bus across all nodes, locks and rate limits on the same port |
  | Sessions | Database | Redis or Valkey | Stateless servers scale out (RNF-005) |
  | Task queue | PostgreSQL | Redis or Valkey, NATS | The same task API whatever the backend |
  | Search | PostgreSQL full-text search | Meilisearch, OpenSearch or Elasticsearch | Search is a port too, not a plugin bolted on |
  | Mail | SMTP | Provider APIs (for example SES) | Templates come from the customisation framework (DEC-008) |

  - Modules depend only on the ports, never on an adapter.
  - Each port has a **conformance suite**: an adapter is supported only when it passes that suite in CI.
  - Plugins (DEC-003) can add adapters.
  - A specialised implementation for one database is allowed only where measurements justify it, and it must pass the same tests.
- Consequences:
  - Queries are no longer checked at compile time by SQLx macros. Typed query builders and the per-database CI matrix make up for it.
  - CI grows by one job per supported backend.
  - The data model stays portable: no database-specific types except behind the port. JSON and full-text search use the port's capabilities, with a fallback where a dialect lacks them.
  - The first release ships PostgreSQL, local and S3 storage, and moka with Redis. The other adapters come in later releases.
- Affected artifacts: DEC-001, `crates/db`, `crates/storage`, `crates/cache`, RNF-016 to RNF-019.
- Refined by: DEC-065 (database port) and DEC-066 (conformance suites and capabilities); port contracts in [infrastructure ports](../../03-design/ports/README.md).
