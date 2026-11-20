---
id: DATA-FILES
status: accepted
origin: requirements
sourceOfTruth: model
---

# Data model: files

PostgreSQL schema `files`, owned by `kampus-files`, following DEC-014, DEC-021, DEC-022 and DEC-026. Rules refer to the [files domain model](../../00-domain-model/files.md). Names are logical; physically the schema carries the installation prefix, `kmp_files` by default (DEC-039).

![Files schema](./files.svg)

## Tables

| Table | Columns | Constraints and indexes |
| --- | --- | --- |
| `blob` | `id`, `hash` bytea, `size` bigint, `media_type` (read from the content when it was first stored), `storage_key` (`sha256/…` in the storage port), `ref_count` integer, `scan_status` (`unscanned`, `pending`, `clean`, `infected`, `failed`), `created_at` | `UNIQUE` (`hash`) (RN-126); `CHECK` `ref_count` >= 0; `unscanned` until the scanning port arrives |
| `area` | `code` PK, `module`, `summary`, `max_size` bigint, `allowed_types` jsonb (exact types or families such as `image/*`), `upload_permission`, `retired_at` | Synchronised from module declarations (DEC-015) |
| `file` | `id`, `blob_id` FK → `blob` (`RESTRICT`), `area_code` FK → `area` (`RESTRICT`), `scope_id` FK → `access.scope` (`CASCADE`, the scope it was uploaded for), `name`, `media_type`, `size`, `uploader` (soft: outlives the account), `attached`, `created_at` | Owned through the owning module's link table (`<module>.<entity>_file`, with foreign keys and `CASCADE` from the owner, RN-127); until `attached`, only its uploader reads it; index on `blob_id` and (`attached`, `created_at`) for the purge |
| `upload` | `id`, `area_code` FK → `area` (`CASCADE`), `scope_id` FK → `access.scope` (`CASCADE`), `uploader`, `name`, `declared_type` (a hint only, RN-130), `max_size`, `blob_id` FK → `blob` (`CASCADE`, once the content arrived), `file_id` FK → `file` (`CASCADE`, once completed), `expires_at` (one hour), `created_at` | Only its uploader sends to it; the purge job removes it after expiry |
| `trash` | `blob_id` PK FK → `blob` (`CASCADE`), `since` | Content no file uses: new content waits here until a file uses it, and content whose last file is detached comes back; the purge job deletes it from storage after 7 days (RN-133) |

Quotas (`quota`, RN-134), conversions (`conversion`, RN-135) and the scanning port (RN-129) arrive in later parts; until then their tables are not created and uploads are not scanned.

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| A file row addressed by context, component, area and item id (polymorphic) | Files owned through the owner module's link table |
| Reference count computed by a query after the delete | Count maintained in the same transaction |
| Each component checking permissions in its own callback | One rule: readable when the owner is readable (RN-128) |
| Tokens that never expire for file access | Signed URLs of a few minutes for one file |
| Upload paths with different checks | One upload pipeline for every client |
