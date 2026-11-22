// ------------------------------------------------------------------------------
//
//     _  __    _    __  __ ____  _   _ ____
//    | |/ /   / \  |  \/  |  _ \| | | / ___|
//    | ' /   / _ \ | |\/| | |_) | | | \___ \
//    | . \  / ___ \| |  | |  __/| |_| |___) |
//    |_|\_\/_/   \_\_|  |_|_|    \___/|____/
//
//    Knowledge Academic Modular Platform for Universal Study
//
// ------------------------------------------------------------------------------
//
// This file is part of Kampus - https://github.com/didactika/kampus
//
// Kampus is free software: you can redistribute it and/or modify it under the terms of the
// GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
// even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with Kampus.
// If not, see <https://www.gnu.org/licenses/>.
//
// @package   web/ui
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


import { LitElement, css, html, nothing } from "lit";
import { customElement, property } from "lit/decorators.js";
import { unsafeSVG } from "lit/directives/unsafe-svg.js";

import { iconOf, onIconsChange } from "../../icons/registry.js";

/**
 * `<k-icon name="delete"></k-icon>`: an icon by name, in the colour of the text around it, as large as the font
 * unless `--k-icon-size` says otherwise. It is decorative unless it has a `label`, which names it for assistive
 * technology. A theme that overrides the name changes it everywhere at once.
 */
@customElement("k-icon")
export class KIcon extends LitElement {
  static override styles = css`
    :host {
      display: inline-flex;
      inline-size: var(--k-icon-size, 1.5rem);
      block-size: var(--k-icon-size, 1.5rem);
      flex: none;
      vertical-align: middle;
      color: inherit;
    }
    span,
    svg {
      display: block;
      inline-size: 100%;
      block-size: 100%;
    }
    svg {
      fill: currentColor;
    }
    .mask {
      background: currentColor;
      mask: var(--src) center / contain no-repeat;
    }
  `;

  /** The name of the icon, as Material Symbols names it (`delete`, `edit`, `visibility`…). */
  @property({ reflect: true }) name = "";

  /** What the icon means, when nothing beside it says so. */
  @property() label = "";

  /** Stops listening to the icons. */
  private unsubscribe: (() => void) | undefined;

  override connectedCallback(): void {
    super.connectedCallback();
    this.unsubscribe = onIconsChange(() => this.requestUpdate());
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback();
    this.unsubscribe?.();
  }

  override render() {
    const source = iconOf(this.name);
    const role = this.label ? "img" : nothing;
    const label = this.label || nothing;
    const hidden = this.label ? nothing : "true";
    if (!source) return html`<span part="icon" role=${role} aria-label=${label} aria-hidden=${hidden}></span>`;
    if ("url" in source) {
      return html`<span
        class="mask"
        part="icon"
        role=${role}
        aria-label=${label}
        aria-hidden=${hidden}
        style="--src: url(${JSON.stringify(source.url)})"
      ></span>`;
    }
    return html`<span part="icon" role=${role} aria-label=${label} aria-hidden=${hidden}>${unsafeSVG(source.svg)}</span>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-icon": KIcon;
  }
}
