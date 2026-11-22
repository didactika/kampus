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

import type { Tone } from "../alert/alert.js";
import { base } from "../../styles/base.js";
import { tones } from "../../styles/tones.js";
import { type } from "../../styles/typography.js";

/** `<k-badge tone="success">Done</k-badge>`: a state or a count, on the container colour of its tone. */
@customElement("k-badge")
export class KBadge extends LitElement {
  static override styles = [
    base,
    tones,
    css`
      :host {
        display: inline-block;
        vertical-align: middle;
      }
      span {
        ${type("label-medium")}
        display: inline-flex;
        align-items: center;
        min-block-size: 1.5rem;
        padding-inline: var(--k-space-2);
        border-radius: var(--k-shape-corner-full);
        background: var(--tone-container);
        color: var(--on-tone-container);
        white-space: nowrap;
      }
    `,
  ];

  /** What it tells; neutral when it says no more than a count. */
  @property({ reflect: true }) tone: Tone | "secondary" | "primary" = "secondary";

  override render() {
    return html`<span part="base"><slot></slot></span>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-badge": KBadge;
  }
}
