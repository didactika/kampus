---
id: FND
status: accepted
origin: requirements
sourceOfTruth: model
---

# Platform foundations

The conventions every module, the API and plugins share: identifiers, time, numbers and money, text and languages, pagination, concurrency, and the shape and versioning of the API. They live in `kampus-common` (DEC-015) and are checked by tests and by the schema linter (DEC-014). Errors follow DEC-018.

## Identifiers (DEC-021)

- Every row has a **UUID version 7** primary key (`uuid` type): unique across campuses and tenants, ordered by creation time, safe to show in URLs and APIs, and impossible to enumerate. No sequential integer ever leaves the database.
- Things people type or read have a **natural key** as well, unique in its scope and enforced by a `UNIQUE` constraint: the short name of a course in its campus, the username in its campus, the code of a permission. Natural keys can change; references always use the UUID.
- External systems keep their identifiers in **external references** (system, external id) attached to the record, never in the record's own columns.
- In Rust, each entity has its own identifier type (`CourseId`, `UserId`…), so an id of one kind cannot be passed where another is expected.

## Time (DEC-022)

- Instants are `timestamptz`, stored in UTC, with microsecond precision. The API exchanges them as RFC 3339 strings with offset (`2026-11-18T10:00:00Z`).
- **Calendar dates** without a time (a birthday, the first day of a term) are `date`; **durations** are a number of seconds or an ISO 8601 duration in the API, never "a timestamp of 0".
- "Never" and "no limit" are `NULL`, never 0 or a magic far-future value.
- Every user has a time zone (IANA name); the campus has a default. Deadlines are instants; recurring events follow RFC 5545 rules and are expanded when read, not stored once per occurrence.
- "Now" comes from one clock in the context, so that tests and scheduled jobs can fix it.

## Numbers and money (DEC-023)

- Exact values are `numeric` with a declared precision: grades `numeric(12,5)`, percentages and weights `numeric(9,5)`, money amounts `numeric(19,4)` with an ISO 4217 currency code. Floating point is never used for grades, weights or money.
- Rounding is explicit and named (half up for display of grades, banker's for money), applied only at the boundaries that need it, and tested.
- Flags are `boolean`; enumerations are checked values or lookup tables, never integer codes without a constraint.

## Text and languages (DEC-024)

- Every user-facing string is a **message** of its module, written in Project Fluent: messages have named parameters, plural and gender rules, and are translated as files per locale (BCP 47 tags: `en`, `es`, `pt-BR`…).
- The terminology layer (DEC-008, DEC-012) can rename the terms a message uses per campus, category, course or section.
- The locale of a request comes from the user, then the campus, then the request headers; right-to-left locales are supported from the start (RNF-010).
- User content is stored as sanitised rich text (one format, sanitised on write and on render) or plain text; the field says which.
- Text is UTF-8 and normalised (NFC); comparisons of natural keys are case-insensitive where people type them.

## Pagination, filtering and sorting (DEC-025)

- Lists are paginated with **opaque cursors** (keyset pagination): `?limit=50&after=<cursor>`, default 50, maximum 200. A response carries `next` and `previous` cursors. Totals are returned only where a list asks for them, and are computed in the same permission-filtered query.
- Filters are query parameters named after the fields (`?status=active&created_after=...`); sorting is `?sort=name,-created_at` over the fields a list declares sortable, with a stable tie-breaker on the id.
- Permissions filter the rows **before** paging (DEC-017).

## Concurrency and repeated requests (DEC-026)

- Every mutable resource has a **version** that increases on each change. The API returns it as an `ETag`; updates and deletes must send `If-Match`, and a stale version is a `precondition_failed` error. No update is ever lost silently.
- Creating requests (`POST`) accept an **`Idempotency-Key`** header: repeating a request with the same key returns the first result instead of creating a duplicate.
- Rules that depend on counts or uniqueness (a maximum number of enrolments, one answer per person) are enforced by the database (constraints, or a lock taken in the same transaction), never by reading first and inserting afterwards.

## API shape and versioning (DEC-025)

- REST over HTTPS, described by an **OpenAPI 3.1** document generated from the code, which is also the source of the web client's typed API client.
- Paths are plural nouns under the major version: `/api/v1/courses/{course_id}/sections`. Actions that are not plain changes are sub-resources (`/api/v1/enrolments/{id}/suspension`).
- JSON bodies use `snake_case` fields, RFC 3339 times, string UUIDs and string decimals for exact numbers.
- Errors are RFC 9457 problem details with the error kind and code of DEC-018.
- Within a major version changes are **additive** only. A deprecated field or endpoint is announced with the `Deprecation` and `Sunset` headers and in the changelog, and kept for at least two minor releases. A breaking change is a new major version, served beside the old one for a transition period.
- Rate limits answer `429` with `Retry-After`; webhooks are signed (HMAC with a per-endpoint secret) and carry an event id for deduplication.

## Project configuration

See [project-configuration.md](./project-configuration.md) for the toolchain, the quality gates and the deployment configuration variables.
