---
id: DEC-081
status: accepted
---

# DEC-081 One semantic version for the platform, a changelog written with every change, and support branches

- Date: 2026-11-18
- Context: Campuses must know exactly what each release changes, what they have to do to upgrade and how long a version gets fixes. Versions scattered across components and change notes written at release time are incomplete and late.
- Options considered: versions per crate; release notes written at release time; one platform version with a changelog maintained in every pull request (chosen).
- Decision:
  - Kampus has **one version** for the server, the web client, the handbook and the container image, following Semantic Versioning: `MAJOR.MINOR.PATCH`. It lives in the workspace manifest and every other place reads it from there. Before 1.0.0 a minor version may break compatibility, always with upgrade steps (DEC-082).
  - Interfaces that third parties depend on carry their **own versions**, also semantic: the HTTP API (`/api/v1`, RN-202), the plugin SDK (RN-211), each event type (RN-137) and each module's schema (its migrations).
  - `CHANGELOG.md` follows Keep a Changelog: an `Unreleased` section on top and one section per release with its date, grouped as Added, Changed, Deprecated, Removed, Fixed and Security, plus an **Upgrade** group with every action an operator must take. A pull request that changes code, configuration, migrations or the container adds its entry; CI refuses it otherwise.
  - Branches: `release/X.Y.Z` integrates the next version; a release merges it into `main` and is tagged `vX.Y.Z`; `support/X.Y` receives the patch releases of a supported minor version. Fixes land on the newest line first and are back-ported.
  - Support: the latest minor version of the current major, and the last minor version of the previous major, receive security and bug fixes; each release states the end of support of the versions it replaces.
  - On a release branch, the workspace version equals the branch version; CI checks it.
- Consequences: Every release has complete notes and upgrade steps from the day it is cut; operators can plan upgrades and support windows.
- Affected artifacts: `Cargo.toml`, `CHANGELOG.md`, `tools/repo-check`, the release process in `CONTRIBUTING.md`, the operator guide.
