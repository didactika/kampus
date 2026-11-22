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
 * `<k-loading label="Loading courses">`: the expressive loading indicator, for waits too short to measure: a shape
 * in the primary colour that turns and changes form. `contained` puts it on a circle of the primary container, for
 * use over content. Use `k-spinner` instead when the wait is long or its progress is known.
 */
@customElement("k-loading")
export class KLoading extends LitElement {
  static override styles = css`
    :host {
      display: inline-grid;
      place-items: center;
      inline-size: var(--k-loading-size, 3rem);
      block-size: var(--k-loading-size, 3rem);
      border-radius: var(--k-shape-corner-full);
    }
    :host([contained]) {
      background: var(--k-color-primary-container);
    }
    span {
      display: block;
      inline-size: 79%;
      block-size: 79%;
      background: var(--k-color-primary);
      animation:
        morph calc(2800ms * var(--k-motion) + 1ms) var(--k-easing-spring-default-spatial) infinite,
        turn calc(4200ms * var(--k-motion) + 1ms) linear infinite;
    }
    :host([contained]) span {
      inline-size: 58%;
      block-size: 58%;
    }
    @keyframes morph {
      0%,
      100% {
        border-radius: 50%;
        scale: 0.9;
      }
      25% {
        border-radius: 30% 70% 70% 30% / 30% 30% 70% 70%;
        scale: 1;
      }
      50% {
        border-radius: 18%;
        scale: 0.86;
      }
      75% {
        border-radius: 62% 38% 46% 54% / 40% 58% 42% 60%;
        scale: 1;
      }
    }
    @keyframes turn {
      to {
        rotate: 360deg;
      }
    }
    @media (prefers-reduced-motion: reduce) {
      span {
        animation: none;
        border-radius: 50%;
      }
    }
  `;

  /** What is being done, for assistive technology. */
  @property() label = "";

  /** Whether it sits on a circle of its own. */
  @property({ type: Boolean, reflect: true }) contained = false;

  override render() {
    return html`<span part="shape" role="status" aria-label=${this.label}></span>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-loading": KLoading;
  }
}
