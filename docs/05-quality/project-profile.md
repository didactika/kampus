---
id: PRF
status: draft
origin: requirements
sourceOfTruth: model
---

# Project profile

| Item | Value |
| --- | --- |
| Project | Kampus (KAMPUS: Knowledge Academic Modular Platform for Universal Study) |
| Documentation language | en |
| Direction | requirements -> design -> code (input: the owner's requirements) |
| Output directory | `docs/` |
| Target stack | Rust, Axum, SQLx, PostgreSQL, cargo test (DEC-001) |
| Created | 2026-11-17 |

## Project objective

Give institutions a complete learning platform (courses, enrolment, roles and permissions, activities, academic records, progress, communication and administration), written from scratch in Rust as a modular system that is safe, fast and customisable, with extras beyond the standard set, and with migration tools for sites and plugins. The core learning entities (course, enrolment, activity, grade) are level 1 of the domain model; identity, permissions and the plugin framework are the structural level that every module depends on.

## Scope

In scope:
- The core functionality of tier 1 of the [requirements](../01-requirements/requirements.md), then the standard activities and services of tier 2.
- A plugin framework with native modules and sandboxed WebAssembly plugins (DEC-003).
- `moodle2kampus`, the plugin converter for migrations (DEC-006).
- The Kampus extras of tier 3, including an importer for sites and course backups (RF-111).

Out of scope (for now):
- Running plugin code of other platforms unconverted inside Kampus.
- Databases other than PostgreSQL in the first release (DEC-001, DEC-009).
- Byte-for-byte compatibility with the database or API of other platforms; the importer and the converter bridge the data and the plugins.

## Assumptions

| ID | Assumption | Needs confirmation from |
| --- | --- | --- |
| A-001 | The first deployments are single institutions on Linux servers or Kubernetes; multi-tenancy is an extra (tier 3), designed in from the start | Owner |
| A-002 | Institutions migrating to Kampus need their courses, users, enrolments and grades moved by the importer | Owner |
| A-003 | The plugins worth converting are those of the currently supported versions of the source platform | Owner |
| A-004 | The product is in English first and internationalised; Spanish is the first additional language | Owner |
