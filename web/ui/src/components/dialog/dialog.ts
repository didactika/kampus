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
import { customElement, property, query } from "lit/decorators.js";

import "../icon/icon.js";
import "../../icons/core.js";
import { base } from "../../styles/base.js";
import { stateLayer } from "../../styles/state-layer.js";
import { type } from "../../styles/typography.js";

/**
 * `<k-dialog heading="Delete the course?" close-label="Close">…<k-button slot="actions">…</k-button></k-dialog>`:
 * a modal dialog on the native `dialog` element. Focus moves into it when it opens and returns where it was when it
 * closes; Escape and the close button close it; the page behind cannot be reached meanwhile.
 */
@customElement("k-dialog")
export class KDialog extends LitElement {
  static override styles = [
    base,
    stateLayer,
    css`
      dialog {
        border: none;
        border-radius: var(--k-shape-corner-extra-large);
        box-shadow: var(--k-elevation-level3);
        background: var(--k-color-surface-container-high);
        color: var(--k-color-on-surface);
        padding: 0;
        inline-size: min(35rem, calc(100vw - 2 * var(--k-space-4)));
        min-inline-size: min(17.5rem, calc(100vw - 2 * var(--k-space-4)));
        max-block-size: calc(100dvh - 2 * var(--k-space-6));
      }
      dialog[open] {
        display: flex;
        flex-direction: column;
        animation: enter var(--k-duration-medium2) var(--k-easing-emphasized-decelerate);
      }
      dialog::backdrop {
        background: color-mix(in srgb, var(--k-color-scrim) 32%, transparent);
      }
      header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: var(--k-space-3);
        padding: var(--k-space-6) var(--k-space-6) var(--k-space-4);
      }
      h2 {
        ${type("headline-small")}
        margin: 0;
      }
      .body {
        ${type("body-medium")}
        color: var(--k-color-on-surface-variant);
        padding-inline: var(--k-space-6);
        overflow-y: auto;
      }
      footer {
        display: flex;
        flex-wrap: wrap;
        justify-content: flex-end;
        gap: var(--k-space-2);
        padding: var(--k-space-6);
      }
      .close {
        display: inline-grid;
        place-items: center;
        font: inherit;
        background: transparent;
        border: none;
        color: var(--k-color-on-surface-variant);
        cursor: pointer;
        inline-size: var(--k-size-control);
        block-size: var(--k-size-control);
        border-radius: var(--k-shape-corner-full);
        margin-block-start: calc(var(--k-space-2) * -1);
        margin-inline-end: calc(var(--k-space-3) * -1);
        flex: none;
      }
      @keyframes enter {
        from {
          opacity: 0;
          transform: translateY(-1rem) scale(0.96);
        }
      }
    `,
  ];

  /** Its title, which names it for assistive technology. */
  @property() heading = "";

  /** The name of the close button, in the language of the page. */
  @property({ attribute: "close-label" }) closeLabel = "Close";

  /** Whether it is open. */
  @property({ type: Boolean, reflect: true }) open = false;

  /** The native dialog. */
  @query("dialog") private dialog!: HTMLDialogElement | null;

  /** What had focus before it opened. */
  private returnTo: Element | null = null;

  /** Opens it. */
  show(): void {
    this.open = true;
  }

  /** Closes it, telling why. */
  close(reason = "close"): void {
    this.open = false;
    this.dispatchEvent(new CustomEvent("k-close", { bubbles: true, composed: true, detail: { reason } }));
  }

  override updated(changed: Map<string, unknown>): void {
    if (!changed.has("open") || !this.dialog) return;
    if (this.open && !this.dialog.open) {
      this.returnTo = document.activeElement;
      this.dialog.showModal();
    } else if (!this.open && this.dialog.open) {
      this.dialog.close();
      if (this.returnTo instanceof HTMLElement) this.returnTo.focus();
    }
  }

  /** Escape closes it through the browser; keeps the state in step. */
  private cancelled(event: Event): void {
    event.preventDefault();
    this.close("escape");
  }

  override render() {
    return html`<dialog part="base" aria-labelledby="heading" @cancel=${this.cancelled}>
      <header>
        <h2 id="heading" part="heading">${this.heading}</h2>
        <button class="close state-layer" part="close" type="button" aria-label=${this.closeLabel} @click=${() => this.close()}>
          <k-icon name="close"></k-icon>
        </button>
      </header>
      <div class="body" part="body"><slot></slot></div>
      <footer part="actions"><slot name="actions"></slot></footer>
    </dialog>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-dialog": KDialog;
  }
}
