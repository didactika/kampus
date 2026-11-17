---
id: DEC-010
status: accepted
---

# DEC-010 Proprietary commercial licence of Didactika, for now

- Date: 2026-11-17
- Context: Q-001 asked which licence Kampus uses.
- Options considered: AGPLv3, GPLv3, Apache-2.0 or MIT, a commercial dual licence, a proprietary commercial licence.
- Decision: For now, Kampus is proprietary software of Didactika under a commercial licence, with all rights reserved (`LICENSE`). The owner will review the licence later.
- Consequences:
  - The repository stays private, and no Kampus code is published or redistributed without Didactika's permission.
  - Third-party dependencies must allow use in proprietary software. `cargo deny` and the npm licence check enforce an allow-list: permissive licences only (MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0 for unmodified files). GPL, AGPL and LGPL dependencies are rejected.
  - Plugins converted by `moodle2kampus` keep their own licence (DEC-006). Most are GPL, so a converted plugin is distributed as a separate work under its own licence and is never merged into the Kampus core.
  - Changing the licence later only needs Didactika's decision, because every contribution is made for Didactika.
- Affected artifacts: `LICENSE`, `README.md`, `deny.toml`, Q-001.
