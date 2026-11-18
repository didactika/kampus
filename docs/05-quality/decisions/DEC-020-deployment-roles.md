---
id: DEC-020
status: proposed
---

# DEC-020 One image with three roles; the web client apart

- Date: 2026-11-18
- Context: Kampus must be easy to install for a small institution and scale for a large one (RNF-005, RNF-013), with zero-downtime minor upgrades (RNF-006).
- Options considered: one process for everything; one image per module; one image whose process takes a role (chosen).
- Decision:
  - One container image of `kampus-server`, started as `server` (HTTP), `worker` (jobs and listeners) or `scheduler` (one active instance, by database lease). A small installation runs the three roles in one process.
  - The web client is a separate, stateless container (SvelteKit server-side rendering) behind the same reverse proxy.
  - State lives only in the ports: database, file storage and cache. Every role is stateless and scales horizontally.
  - Deployment configuration is a file plus `KAMPUS_*` environment variables, validated at start-up for the role being started.
  - Migrations follow expand and contract, so a minor upgrade rolls instance by instance.
- Consequences: one artefact to build, sign and scan (RNF-014); Docker Compose for development and small installations, a Helm chart for clusters.
- Affected artifacts: `crates/kampus-server`, `web/`, the Docker and Helm files, the operator guide.
