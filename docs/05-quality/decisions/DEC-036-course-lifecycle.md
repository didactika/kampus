---
id: DEC-036
status: accepted
---

# DEC-036 Course lifecycle with archiving as a freeze, and deletion as a cascading job

- Date: 2026-11-18
- Context: Courses run for a term, are kept as a record, and are eventually removed. Deleting a course by code table by table leaves orphans when a step fails.
- Options considered: a visible flag only; manual archiving by hiding; a lifecycle with archive as a frozen scope and deletion as a job relying on cascades (chosen).
- Decision:
  - States draft, published, archived, deletion scheduled; archiving freezes the course scope (read-only for everyone).
  - Deletion is a scheduled job with a grace period; owned data is removed by foreign keys with ON DELETE CASCADE and the privacy handlers of every module.
  - Course owners are data; deletion needs `courses.course.delete`, which owners hold through the owner role.
  - Deleted sections go to a recycle bin until it expires.
- Consequences: Closing a term is one action that keeps everything readable; deletions are complete and recoverable during the grace period.
- Affected artifacts: `crates/kampus-courses`, every module with course data, the operator guide.
