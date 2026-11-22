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
import { customElement, property, state } from "lit/decorators.js";
import { repeat } from "lit/directives/repeat.js";

import type { Tone } from "../alert/alert.js";
import { base } from "../../styles/base.js";
import { stateLayer } from "../../styles/state-layer.js";
import { type } from "../../styles/typography.js";

/** One message of the toast region. */
interface Toast {
  /** Its key. */
  readonly id: number;
  /** What it says. */
  readonly message: string;
  /** What it tells. */
  readonly tone: Tone;
}

let next = 0;

/**
 * `<k-toasts>`: the region at the foot of the page where the results of asynchronous actions appear, each a short
 * message on the inverse surface. It is a live region, so each message is read out without moving focus; a message
 * leaves after a while, or at once with its close button.
 */
@customElement("k-toasts")
export class KToasts extends LitElement {
  static override styles = [
    base,
    stateLayer,
    css`
      :host {
        position: fixed;
        inset-block-end: var(--k-space-4);
        inset-inline: var(--k-space-4);
        z-index: var(--k-layer-toast);
        display: flex;
        justify-content: center;
        pointer-events: none;
      }
      div {
        display: contents;
      }
      ol {
        list-style: none;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: var(--k-space-2);
        inline-size: min(42rem, 100%);
      }
      li {
        ${type("body-medium")}
        pointer-events: auto;
        display: flex;
        align-items: center;
        gap: var(--k-space-2);
        min-block-size: 3rem;
        padding-block: var(--k-space-1);
        padding-inline: var(--k-space-4) var(--k-space-1);
        background: var(--k-color-inverse-surface);
        color: var(--k-color-inverse-on-surface);
        border-radius: var(--k-shape-corner-extra-small);
        box-shadow: var(--k-elevation-level3);
        border-inline-start: 4px solid var(--mark);
        animation: rise var(--k-duration-medium2) var(--k-easing-emphasized-decelerate);
      }
      .message {
        flex: 1;
        padding-block: var(--k-space-2);
      }
      .info,
      .success,
      .warning,
      .danger {
        --mark: transparent;
      }
      .danger {
        --mark: var(--k-color-danger-container);
      }
      .warning {
        --mark: var(--k-color-warning-container);
      }
      button {
        display: inline-grid;
        place-items: center;
        font: inherit;
        inline-size: var(--k-size-control);
        block-size: var(--k-size-control);
        border: none;
        border-radius: var(--k-shape-corner-full);
        background: transparent;
        color: inherit;
        cursor: pointer;
        flex: none;
      }
      @keyframes rise {
        from {
          opacity: 0;
          transform: translateY(1rem);
        }
      }
    `,
  ];

  /** How long a message stays, in milliseconds. */
  static lifetime = 6000;

  /** The name of the close button, in the language of the page. */
  @property({ attribute: "close-label" }) closeLabel = "Dismiss";

  /** The messages showing. */
  @state() private toasts: Toast[] = [];

  /** Shows a message. */
  push(message: string, tone: Tone = "info"): void {
    const toast = { id: ++next, message, tone };
    this.toasts = [...this.toasts, toast];
    setTimeout(() => this.dismiss(toast.id), KToasts.lifetime);
  }

  /** Removes a message. */
  dismiss(id: number): void {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  override render() {
    return html`<div role="status" aria-live="polite" aria-atomic="false"><ol part="base">
      ${repeat(
        this.toasts,
        (t) => t.id,
        (t) =>
          html`<li class=${t.tone} part="toast">
            <span class="message">${t.message}</span>
            <button class="state-layer" type="button" aria-label=${this.closeLabel} @click=${() => this.dismiss(t.id)}>
              <span aria-hidden="true">✕</span>
            </button>
          </li>`,
      )}
    </ol></div>`;
  }
}

/** Shows a message in the toast region of the page, creating the region the first time. */
export function toast(message: string, tone: Tone = "info"): void {
  let region = document.querySelector("k-toasts");
  if (!region) {
    region = document.createElement("k-toasts");
    document.body.append(region);
  }
  region.push(message, tone);
}

declare global {
  interface HTMLElementTagNameMap {
    "k-toasts": KToasts;
  }
}
