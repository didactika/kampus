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
| `blob` | `id`, `hash` bytea, `size` bigint, `media_type`, `storage_key`, `ref_count` integer, `scan_status` (`pending`, `clean`, `infected`, `failed`), `created_at` | `UNIQUE` (`hash`); `CHECK` `ref_count` >= 0 |
| `area` | `code` PK, `module`, `max_size` bigint, `allowed_types` text[], `quota_class` | Synchronised from module declarations (DEC-015) |
| `file` | `id`, `blob_id` FK → `blob` (`RESTRICT`), `area_code` FK → `area`, `name`, `media_type`, `size`, `uploaded_by` FK → `identity.account`, `created_at` | Owned through the owning module's link table (`<module>.<entity>_file`, with foreign keys and `CASCADE` from the owner) |
| `upload_session` | `id`, `area_code` FK → `area`, `account_id` FK → `identity.account`, `max_size`, `token_hash` `UNIQUE`, `expires_at`, `completed_file_id` FK → `file` | Purged by a job after expiry |
| `quota` | `scope_id` FK → `access.scope`, `limit_bytes` bigint, `used_bytes` bigint | PK (`scope_id`); `used_bytes` maintained in the attaching transaction |
| `conversion` | `id`, `source_blob_id` FK → `blob` (`CASCADE`), `kind`, `result_blob_id` FK → `blob`, `state`, `error` | `UNIQUE` (`source_blob_id`, `kind`) |
| `trash` | `blob_id` PK FK → `blob` (`CASCADE`), `since` | Blobs with no references, deleted from storage after retention |

## Anti-patterns avoided (DEC-014)

| Avoided | How |
| --- | --- |
| A file row addressed by context, component, area and item id (polymorphic) | Files owned through the owner module's link table |
| Reference count computed by a query after the delete | Count maintained in the same transaction |
| Each component checking permissions in its own callback | One rule: readable when the owner is readable (RN-128) |
| Tokens that never expire for file access | Signed URLs of a few minutes for one file |
| Upload paths with different checks | One upload pipeline for every client |
