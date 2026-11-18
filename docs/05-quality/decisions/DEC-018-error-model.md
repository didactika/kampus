---
id: DEC-018
status: proposed
---

# DEC-018 One error model, problem details on the API

- Date: 2026-11-18
- Context: errors cross module boundaries and reach the web client, the API, plugins and logs. Strings and ad hoc codes make them impossible to translate, test or handle.
- Options considered: one error type per crate with conversions; free-form messages; one shared error model with a closed set of kinds (chosen).
- Decision:
  - `kampus-common` defines the kinds: `not_found`, `permission_denied`, `validation` (field and rule), `conflict`, `precondition_failed`, `rate_limited`, `unavailable`, `internal`. Each error carries a stable code of its module and translatable parameters.
  - The API maps them to HTTP statuses and RFC 9457 problem details. Internal errors expose only the request id, never a stack trace or data.
  - Validation errors list every failing field at once.
- Consequences: clients and plugins handle errors by kind and code; messages are translated like any other string (RNF-010); logs keep the full cause with the request id.
- Affected artifacts: `crates/kampus-common`, `crates/kampus-api`, the OpenAPI document, the web client.
