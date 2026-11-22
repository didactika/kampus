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


import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";

/**
 * `<k-button-group label="View"><k-button>List</k-button><k-button>Grid</k-button></k-button-group>`: buttons that
 * belong together. Connected, they touch with small corners between them and full corners at the ends, as one
 * control; standard, they sit apart with a small gap.
 */
@customElement("k-button-group")
export class KButtonGroup extends LitElement {
  static override styles = css`
    :host {
      display: inline-flex;
      flex-wrap: wrap;
      align-items: center;
      gap: var(--k-space-2);
    }
    :host([connected]) {
      flex-wrap: nowrap;
      gap: 2px;
    }
    :host([connected]) ::slotted(k-button) {
      --k-button-start-radius: var(--k-shape-corner-small);
      --k-button-end-radius: var(--k-shape-corner-small);
    }
    :host([connected]) ::slotted(k-button:first-child) {
      --k-button-start-radius: var(--k-shape-corner-full);
    }
    :host([connected]) ::slotted(k-button:last-child) {
      --k-button-end-radius: var(--k-shape-corner-full);
    }
  `;

  /** What the buttons are about, for assistive technology. */
  @property() label = "";

  /** Whether the buttons touch as one control. */
  @property({ type: Boolean, reflect: true }) connected = false;

  override connectedCallback(): void {
    super.connectedCallback();
    this.setAttribute("role", "group");
  }

  override updated(): void {
    if (this.label) this.setAttribute("aria-label", this.label);
    else this.removeAttribute("aria-label");
  }

  override render() {
    return html`<slot></slot>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-button-group": KButtonGroup;
  }
}
