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

/** `<k-spinner label="Loading courses">`: a circular indicator of work in progress, announced by its label. */
@customElement("k-spinner")
export class KSpinner extends LitElement {
  static override styles = [
    base,
    css`
      :host {
        display: inline-block;
        inline-size: 3rem;
        block-size: 3rem;
      }
      span {
        display: block;
        inline-size: 100%;
        block-size: 100%;
      }
      svg {
        display: block;
        inline-size: 100%;
        block-size: 100%;
        animation: turn calc(1400ms * var(--k-motion) + 1ms) linear infinite;
      }
      .track {
        stroke: var(--k-color-surface-container-highest);
      }
      .arc {
        stroke: var(--k-color-primary);
        stroke-linecap: round;
        stroke-dasharray: 80 200;
        animation: grow calc(1400ms * var(--k-motion) + 1ms) var(--k-easing-standard) infinite;
      }
      @keyframes turn {
        to {
          transform: rotate(360deg);
        }
      }
      @keyframes grow {
        0% {
          stroke-dasharray: 10 200;
          stroke-dashoffset: 0;
        }
        50% {
          stroke-dasharray: 90 200;
          stroke-dashoffset: -30;
        }
        100% {
          stroke-dasharray: 10 200;
          stroke-dashoffset: -125;
        }
      }
    `,
  ];

  /** What is being done, for assistive technology. */
  @property() label = "";

  override render() {
    return html`<span part="base" role="status" aria-label=${this.label}>
      <svg viewBox="0 0 48 48" aria-hidden="true" fill="none" stroke-width="4">
        <circle class="track" cx="24" cy="24" r="20"></circle>
        <circle class="arc" cx="24" cy="24" r="20"></circle>
      </svg>
    </span>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-spinner": KSpinner;
  }
}
