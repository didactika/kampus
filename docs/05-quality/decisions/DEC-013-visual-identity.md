---
id: DEC-013
status: accepted
---

# DEC-013 Default visual identity: Didactika's colours, customisable in settings

- Date: 2026-11-17
- Context: The owner requires Kampus to ship with Didactika's colours as its base, and every campus to be able to change them in its settings. The source of the colours is Didactika's design system (its brand tokens).
- Decision: the default theme of Kampus uses Didactika's brand tokens. Every value is a design token (DEC-004) that the customisation framework (DEC-008) lets administrators override in settings, per campus, tenant or course category, with no code.

  | Token | Default | Source |
  | --- | --- | --- |
  | `brand` | `#00555a` | Didactika logo teal |
  | `primary` | `#0f766e` | teal 700 |
  | `primary-scale` | teal 50 `#f0fdfa`, 100 `#ccfbf1`, 200 `#99f6e4`, 300 `#5eead4`, 400 `#2dd4bf`, 500 `#14b8a6`, 600 `#0d9488`, 700 `#0f766e`, 800 `#115e59`, 900 `#134e4a`, 950 `#042f2e` | Didactika teal scale |
  | `accent` | `#06b6d4` | cyan 500 |
  | `secondary` | `#475569` | slate 600 |
  | `success` | `#15803d` | role |
  | `info` | `#0369a1` | role |
  | `warning` | `#b45309` | role |
  | `danger` | `#b91c1c` | role |
  | `surface-light` / `surface-dark` | `#f8fafc` / `#0f172a` | slate 50 / slate 900 |

  - **Icon.** The Kampus icon (`docs/assets/kampus-icon.svg`) is a K made of three rounded strokes: a white stem and upper arm, and a lower arm with a teal-to-cyan gradient for progress. It sits on a square with rounded corners and a teal gradient from `#0f766e` through `#00555a` to `#042f2e`.
  - **Customisation.** In settings, a campus can replace the logo and the icon, and change the seed colours, the typefaces, the size, the corner radius, the density, the motion and the preferred scheme, with a live preview. The colour roles of the light and the dark scheme are grown from the seeds in the manner of Material 3. A role keeps its seed when it is accessible and the nearest accessible tone of its palette otherwise, so no seed can break WCAG 2.2 AA contrast. Didactika's palettes above are used as they are; any other seed grows a palette in OKLCH that keeps its hue.
- Consequences:
  - The design system of the web client and the handbook (DEC-011) use these tokens from the start.
  - The no-code theme editor (RF-115) edits the same tokens.
  - When Didactika's palette changes, the default theme follows it.
- Affected artifacts: `docs/assets/kampus-icon.svg`, `web/` design system, `handbook/`, RF-006, RF-115.
