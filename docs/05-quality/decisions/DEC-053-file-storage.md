---
id: DEC-053
status: accepted
---

# DEC-053 Content-addressed blobs, owned files, permission through the owner and signed URLs

- Date: 2026-11-18
- Context: Files are attached to almost everything; polymorphic owners, reference counts computed after the fact and per-component permission callbacks lead to orphaned content, leaks and duplicated checks.
- Options considered: one polymorphic file table addressed by owner type and id; files stored inside each module; content-addressed blobs with files owned through each module's link table (chosen).
- Decision:
  - Blobs are stored once per content hash; the reference count changes in the transaction that attaches or detaches a file.
  - A file belongs to one owner through the owner module's link table with foreign keys; deleting the owner removes the link and decrements the count.
  - A file is readable exactly when its owner is readable; the owning module declares the area's resolver.
  - Content is served through signed URLs valid for minutes, for one file; active content from a separate origin.
  - Unreferenced blobs go to a trash and are deleted by a job after retention.
- Consequences: No orphaned files, no polymorphic owner, one permission rule; storage backends can be swapped by a verified copy job.
- Affected artifacts: `crates/kampus-files`.
