---
id: DEC-002
status: accepted
---

# DEC-002 Modular monolith, one crate per module

- Date: 2026-11-17
- Context: Kampus must be modular, easy to deploy and with boundaries the compiler enforces.
- Options considered: microservices; one crate; a Cargo workspace with one crate per module.
- Decision: A modular monolith: one Cargo workspace, one crate per module of the components diagram, a `kampus-server` binary that composes them. Modules expose a public API and own their tables; dependencies follow the domain ranking.
- Consequences: Cargo forbids cycles between crates, so the module graph stays acyclic; a module can be extracted into a service later if needed.
- Affected artifacts: `docs/03-design/components/`, `crates/`.
