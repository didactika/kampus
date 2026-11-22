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

import { base } from "../../styles/base.js";
import { tones } from "../../styles/tones.js";
import { type } from "../../styles/typography.js";

/** What a message tells: it informs, something is done, something needs attention, or something failed. */
export type Tone = "info" | "success" | "warning" | "danger";

/**
 * `<k-alert tone="danger" heading="Not saved">…</k-alert>`: a message in the page, on the container colour of its
 * tone. A danger or a warning is announced at once; information and success politely.
 */
@customElement("k-alert")
export class KAlert extends LitElement {
  static override styles = [
    base,
    tones,
    css`
      :host {
        display: block;
      }
      .alert {
        ${type("body-medium")}
        display: grid;
        gap: var(--k-space-1);
        background: var(--tone-container);
        color: var(--on-tone-container);
        border-radius: var(--k-shape-corner-medium);
        padding-block: var(--k-space-3);
        padding-inline: var(--k-space-4);
      }
      .heading {
        ${type("title-small")}
        margin: 0;
      }
    `,
  ];

  /** What it tells. */
  @property({ reflect: true }) tone: Tone = "info";

  /** Its title. */
  @property() heading = "";

  override render() {
    const urgent = this.tone === "danger" || this.tone === "warning";
    return html`<div class="alert" part="base" role=${urgent ? "alert" : "status"}>
      ${this.heading ? html`<p class="heading" part="heading">${this.heading}</p>` : null}
      <div><slot></slot></div>
    </div>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-alert": KAlert;
  }
}
