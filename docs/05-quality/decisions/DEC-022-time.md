---
id: DEC-022
status: accepted
---

# DEC-022 Instants in UTC, dates and durations apart, NULL for never

- Date: 2026-11-18
- Context: Deadlines, availability, sessions, logs and calendars all depend on time; integer timestamps, zero sentinels and local times make every comparison a bug waiting to happen.
- Options considered: integer seconds; local times; timestamptz in UTC with typed dates and durations (chosen).
- Decision:
  - Instants are timestamptz in UTC with microsecond precision, exchanged as RFC 3339.
  - Calendar dates are date; durations are seconds or ISO 8601 durations.
  - "Never" and "no limit" are NULL, never 0 or a far-future value.
  - Users and campuses have IANA time zones; recurring events follow RFC 5545 and are expanded when read.
  - The current time comes from a clock in the context.
- Consequences: Deadlines are unambiguous across time zones and daylight saving; tests and jobs can fix the clock; no sentinel values in the schema.
- Affected artifacts: `crates/common`, every schema, the API, the calendar.
