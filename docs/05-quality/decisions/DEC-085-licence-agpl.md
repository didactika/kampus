---
id: DEC-085
status: accepted
---

# DEC-085 Kampus is free software under the GNU Affero General Public License, version 3 or later

- Date: 2026-11-18
- Context: DEC-010 kept Kampus proprietary while the owner reviewed the licence (Q-001). The owner chose the most protective copyleft: whoever distributes Kampus, a modified version or a part of it, or offers it to users over a network, must give them the complete source under the same licence.
- Options considered: a permissive licence; GPLv3, whose obligations start only with distribution, so a modified Kampus offered as a service could stay closed; AGPLv3 or later (chosen).
- Decision:
  - Kampus is licensed under the **GNU Affero General Public License, version 3 or (at your option) any later version** (`AGPL-3.0-or-later`); `LICENSE` holds its text and every file carries the notice in its header (DEC-083).
  - The copyright holder is Didactika; contributions are made under the same licence.
  - Dependencies must be compatible with AGPLv3: permissive licences (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0, CDLA-Permissive-2.0), MPL-2.0, LGPL-2.1-or-later, LGPL-3.0, GPL-3.0 and AGPL-3.0. GPL-2.0-only and every licence incompatible with GPLv3 are rejected. `cargo deny` enforces the list.
  - The documentation (`didactika/kampus-docs`) carries the same licence.
  - Plugins are separate works when they only use the SDK interfaces; plugins converted by `moodle2kampus` (GPL) are now license-compatible with the platform.
- Consequences: Kampus and every derivative stay free, including when offered as a hosted service; the repositories can be made public whenever the owner decides.
- Affected artifacts: `LICENSE`, every file header, `Cargo.toml`, `deny.toml`, `README.md`, DEC-010, Q-001, `didactika/kampus-docs`.
