---
id: DEC-066
status: accepted
---

# DEC-066 One conformance suite per port, capabilities instead of backend checks

- Date: 2026-11-18
- Context: Interchangeable adapters are only interchangeable if they behave the same; code that checks which backend it runs on spreads backend details through the modules.
- Options considered: tests per adapter; trusting the backend's documentation; one suite per port and declared capabilities (chosen).
- Decision:
  - Each port has one conformance suite written against its trait; every adapter runs it in CI and is released only when green.
  - Ports declare capabilities; modules ask for a capability and use its fallback, never the adapter's name.
  - A module that requires a capability declares it; start-up refuses a configuration that lacks it.
  - Adapters live in their own crates and report the same metrics.
- Consequences: Adding a backend is bounded work with a clear finish line; modules stay free of backend details.
- Affected artifacts: `crates/db`, `crates/storage`, `crates/cache`, the CI matrix, RNF-016 to RNF-019.
