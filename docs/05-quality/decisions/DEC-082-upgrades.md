---
id: DEC-082
status: accepted
---

# DEC-082 Upgrade from any supported release to any later one in one run, without breaking what depends on Kampus

- Date: 2026-11-18
- Context: Upgrading through every intermediate release is slow and error-prone, and breaking an interface forces integrations, plugins and campuses to change at the same time as the platform.
- Options considered: upgrades only from the previous release; dumps and re-imports between majors; cumulative, ordered upgrade steps kept for every supported release, with backward-compatible interfaces (chosen).
- Decision:
  - **Cumulative upgrades.** Each module keeps all its released migrations, ordered and immutable (checksums, DEC-080). `kampus-server upgrade` (and the start of the server, when allowed) applies every pending step of every module in dependency order, followed by the data upgrade jobs, so any supported release reaches the target release in one run. Intermediate releases are never needed.
  - **Recorded state.** The database records each module's applied steps with the release that shipped them; the server refuses to start when its code is older than the schema, or when a step it knows was changed.
  - **Baselines.** A new major may add a baseline for fresh installations, but keeps the steps needed to upgrade from the supported releases of the previous major (DEC-081).
  - **Backward compatibility.** Within a major version the API, the plugin SDK, event payloads (new versions with upcasters for listeners of the old one), settings keys and configuration variables only change additively. Anything removed is first deprecated, kept working, announced in the changelog and removed in the next major, with its upgrade step.
  - **Tested paths.** CI upgrades a database of every supported release to the current code and checks it against a fresh installation of the same version.
- Consequences: Campuses can skip releases; integrations and plugins keep working across minor versions; every incompatible change comes with its way forward.
- Affected artifacts: `crates/db`, `crates/server`, every module's migrations, CI, the operator guide.
