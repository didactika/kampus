# Audit

Date: 2026-11-18. Scope: the design of tier 1 (phase B review): requirements, use cases, domain models, data models, API fragments, decisions, highlights, the plugin SDK, the web client, the handbook and the infrastructure ports.

Result: **no blocking findings**. Every finding below is fixed except AUD-B12 and AUD-B13, which are minor and planned.

## Coverage

| Item | Count | Traced |
| --- | --- | --- |
| Functional requirements of tier 1 (Must) | every one | to a use case, or to the architecture traceability table for the cross-cutting ones (RF-001, RF-003, RF-007, RF-008); RF-101 is designed with the migration tool (AUD-B12) |
| Use cases | 60 (UC-001 to UC-060) | four cards each (requirements, analysis, design, implementation), a sequence, a wireframe, and a row in [traceability.csv](./traceability.csv) with its code and test locations |
| Business rules | RN-001 to RN-222 | each defined once, in its module's domain model, and covered by at least one use case |
| Non-functional requirements | RNF-001 to RNF-021 | each traced in the [architecture](../03-design/architecture.md#traceability) |
| Decisions | DEC-001 to DEC-080 | all accepted; every reference points to an existing record |
| Highlights | HL-001 to HL-069 | every one cites existing decisions |
| Schema dependencies | 19 schemas | no foreign key to a later module (checked in CI) |
| API fragments | 16 | valid OpenAPI 3.1 (checked in CI) |

## Findings

| ID | Severity | Finding | Evidence | Impact | Fix | Status |
| --- | --- | --- | --- | --- | --- | --- |
| AUD-B01 | major | Eleven foreign keys pointed to modules later in the ranking, including a cycle between `access` and `identity` | data models of access, identity, settings, i18n, customise, activities | modules could not be built and migrated in order | `access.principal`, new level 1 order, customisation at level 2, grade conditions in records (DEC-079) | fixed (#32) |
| AUD-B02 | minor | The person's language was kept both in the profile and in the languages module | `users.profile`, `i18n.language_choice` | two sources for one fact | one column in the profile, one in the course (DEC-079) | fixed (#32) |
| AUD-B03 | minor | Profile fields duplicated the custom fields of customisation | `users.profile_field` | two field engines | profile fields are custom fields of kind `account` | fixed (#32) |
| AUD-B04 | major | DEC-014 asked for reversible migrations, DEC-065 for forward-only ones | decision records | contradictory rule for every module | forward-only with expand and contract (DEC-080) | fixed (#32) |
| AUD-B05 | minor | The data model overview was a template placeholder | `03-design/data-model/data-model.puml` | no overview of the schemas | generated from the data models | fixed (#32) |
| AUD-B06 | minor | Every use case wireframe was a placeholder | `01-requirements/use-cases/*/wireframes.puml` | screens not specified | generated from each use case's flows inside the shell | fixed (#31) |
| AUD-B07 | major | The plugin storage interface took SQL text filters, against the rule of no raw SQL for plugins | `03-design/sdk/kampus.wit` | sandbox bypass | structured conditions | fixed (#30) |
| AUD-B08 | major | Custom field values carried a polymorphic owner id | `customise.field_value` | the anti-pattern DEC-014 forbids | link tables in the owning modules (DM-03) | fixed (#30) |
| AUD-B09 | major | Eleven Must non-functional requirements were not traced in the architecture | `03-design/architecture.md` | no evidence of where they are met | traceability table completed | fixed (this review) |
| AUD-B10 | minor | Twenty-three business rules were covered by no use case | domain models | rules without a scenario or a test | added to the use cases that exercise them | fixed (this review) |
| AUD-B11 | major | Thirty summaries and descriptions of the API fragments broke the OpenAPI schema (unquoted commas turned into extra keys) | `03-design/api/*.openapi.yaml` | invalid documents | quoted; OpenAPI validation added to CI | fixed (this review) |
| AUD-B12 | minor | RF-101 (the plugin migration tool) has no use case yet | `01-requirements/requirements.md` | not part of the tier 1 platform | designed with the tool, after the plugin SDK it targets (`docs/03-design/sdk/`) | planned |
| AUD-B13 | minor | The API fragments have no operation ids, security schemes or declared path parameters | lint warnings of the fragments | the published document needs them | the document is generated from the routes the modules declare (RN-195), which carry them | planned |

## Checks executed

| Check | Tool | Result |
| --- | --- | --- |
| Links, anchors and diagram rendering | `tools/docs-check` | pass |
| Foreign keys only to earlier modules | `tools/docs-check` (schema dependencies) | pass |
| API fragments against OpenAPI 3.1 | Redocly CLI, minimal rules | pass, warnings listed in AUD-B13 |
| Plugin SDK WIT packages | `wasm-tools component wit` | pass |
| Requirement, use case, rule, decision and highlight traceability | review script over `docs/` | pass |

## Checks not executed

| Check | Why |
| --- | --- |
| Visual inspection of every rendered wireframe | Wireframes render without errors; their layout is reviewed with each module's screens in phase C |
| Code and test locations in the traceability table | They are the planned locations; no code exists before phase C |
| Query plans of the key use cases (DEC-014 rule 8) | They need the implemented schema and data |
