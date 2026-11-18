---
id: DEC-056
status: accepted
---

# DEC-056 Declared notification types, typed preferences, delivery jobs and templates

- Date: 2026-11-18
- Context: Preferences kept as concatenated keys and comma lists cannot be validated; sending mail inside the request slows it and loses messages when the mail server fails.
- Options considered: preferences as free settings; one notification table per module; declared types with typed preference rows and one delivery job per recipient and channel (chosen).
- Decision:
  - Modules declare notification types; required types cannot be turned off, only routed.
  - Preferences are rows per person, type and channel; the campus sets defaults and may lock them.
  - Notifications are created by listeners after commit; each recipient and channel is one job with retries.
  - Texts come from templates per scope and locale, using the campus terminology.
- Consequences: Fast requests, no lost notifications, preferences that can be explained.
- Affected artifacts: `crates/notify`.
