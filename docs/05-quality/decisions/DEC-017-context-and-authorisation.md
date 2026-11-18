---
id: DEC-017
status: accepted
---

# DEC-017 Request context, and permissions checked inside every service operation

- Date: 2026-11-18
- Context: when permission checks live in pages or handlers, every new entry point (an API endpoint, a mobile route, a plugin, a report) has to repeat them, and some will not. Lists filtered after paging leak counts and miss rows.
- Options considered: checks in HTTP handlers; checks in a middleware by route; checks inside each service operation, against the scope of the object it touches (chosen).
- Decision:
  - Every service operation receives a **context**: actor (user, API client, job or plugin), campus and tenant, locale, time zone, request id and trace.
  - The operation itself checks the permission against the **scope of the object** it reads or changes (DEC-007). Handlers, jobs and plugins call the same operation and cannot skip the check.
  - List operations filter by permission **in the query**, before paging.
  - Jobs and listeners run with the context of the actor that caused them, or with an explicit system actor that is recorded.
  - Sessions are secure, HTTP-only cookies; API tokens are scoped, expire and are stored hashed; secrets come from a cryptographically secure generator.
- Consequences: one rule for the web client, the API and plugins; permission explanations (RF-023) and "view as" (HL-014) use the same check; services need the context as an argument, which tests build with helpers.
- Affected artifacts: `crates/common` (context), `crates/access`, every service operation, `crates/api`, the plugin SDK.
