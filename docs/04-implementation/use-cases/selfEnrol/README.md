---
id: UC-017
name: selfEnrol
title: Enrol oneself in a course
status: accepted
origin: requirements
sourceOfTruth: model
---

# UC-017 Enrol oneself in a course — implementation

| Overview | Requirements | Analysis | Design | Implementation | Tests |
| --- | --- | --- | --- | --- | --- |
| [Use cases](../../../01-requirements/use-case-overview.svg) | [Requirements](../../../01-requirements/use-cases/selfEnrol/README.md) | [Analysis](../../../02-analysis/use-cases/selfEnrol/README.md) | [Design](../../../03-design/use-cases/selfEnrol/README.md) | **Implementation** | [Tests](../../../04-implementation/use-cases/selfEnrol/README.md#tests) |

Design: [README](../../../03-design/use-cases/selfEnrol/README.md)

## Code

| Element | Kind | Location | Verified |
| --- | --- | --- | --- |
| Self-enrolment service | service | `crates/kampus-enrol/src/self_enrol.rs` | no (to build) |
| `POST /api/v1/enrolment-instances/{instance_id}/self` | API endpoint | `docs/03-design/api/enrolment.openapi.yaml` | no (to build) |
| `enrol.self_key`, `enrol.request`, `enrol.self_attempt` | tables | `docs/03-design/data-model/enrolment.md` | no (to build) |

## Tests

| Test | Covers | Location | Result |
| --- | --- | --- | --- |
| Unit and integration tests of the module | RN-063 to RN-066 | `crates/kampus-enrol/tests/` | planned |
