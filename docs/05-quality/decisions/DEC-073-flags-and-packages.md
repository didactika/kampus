---
id: DEC-073
status: accepted
---

# DEC-073 Declared feature flags by scope and customisation packages with previewed imports

- Date: 2026-11-18
- Context: Features switched by hidden settings and customisations copied by hand between campuses lead to drift and surprises.
- Options considered: settings per feature; database copies; declared flags and validated packages (chosen).
- Decision:
  - Modules declare feature flags with defaults; a disabled feature hides its routes, UI modules and contributions in that scope and keeps its data.
  - Customisations are exported as versioned packages and imported after a preview of the differences, in one transaction recorded as one change.
- Consequences: Features can be rolled out per campus, tenant or course (RF-118); campuses share configurations safely.
- Affected artifacts: `crates/kampus-customise`, the web client.
