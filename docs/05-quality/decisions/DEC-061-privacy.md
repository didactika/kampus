---
id: DEC-061
status: accepted
---

# DEC-061 Privacy requests handled by declared per-module handlers, retention by data category

- Date: 2026-11-18
- Context: Exports and deletions that depend on a central list of modules miss data; deletion that removes the account row breaks records that must be kept; retention set per screen is impossible to audit.
- Options considered: a central privacy module that knows every table; each module on its own; declared handlers and data categories collected at start-up (chosen).
- Decision:
  - Each module with personal data declares a privacy handler and its data categories; the build fails otherwise.
  - Requests are split into one job per module; deletions are approved unless the campus allows self-service; legal holds suspend them.
  - Deletion produces the tombstone of RN-034; exports are files available for a limited time through signed URLs.
  - Retention rules are per data category and campus, applied by jobs that record what they erased.
  - Policies are versioned; required ones are accepted again when they change.
- Consequences: GDPR duties (RNF-002) are met by construction for every module and plugin.
- Affected artifacts: `crates/privacy`, every module's privacy handler.
