---
id: DEC-057
status: accepted
---

# DEC-057 Messaging with unique direct conversations, campus rules and retention

- Date: 2026-11-18
- Context: Direct conversations de-duplicated by read-then-insert end up duplicated under concurrency; messages deleted by everyone kept forever are a privacy liability.
- Options considered: one conversation per message thread; a unique key on the ordered pair of people (chosen).
- Decision:
  - A direct conversation is unique per ordered pair; course group conversations are unique per group.
  - Who may message whom is a campus rule (shared course, contact, role), plus blocks.
  - Deletion is per member; a job purges messages deleted by every member or older than the retention policy.
  - Bodies are sanitised rich text; attachments use the conversation's file area.
- Consequences: No duplicate conversations; deleted content really goes away.
- Affected artifacts: `crates/notify`.
