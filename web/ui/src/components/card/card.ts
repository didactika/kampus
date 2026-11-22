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

import { base } from "../../styles/base.js";
import { type } from "../../styles/typography.js";

/** How a card stands out: with a line around it, raised over the page, or filled with a deeper surface. */
export type CardVariant = "outlined" | "elevated" | "filled";

/** `<k-card heading="Physics">…<k-button slot="actions">…</k-button></k-card>`: a block of related content. */
@customElement("k-card")
export class KCard extends LitElement {
  static override styles = [
    base,
    css`
      :host {
        display: block;
      }
      section {
        display: flex;
        flex-direction: column;
        gap: var(--k-space-3);
        block-size: 100%;
        border-radius: var(--k-shape-corner-medium);
        padding: var(--k-space-4);
        transition: box-shadow var(--k-duration-short4) var(--k-easing-standard);
      }
      .outlined {
        background: var(--k-color-surface-container-lowest);
        border: 1px solid var(--k-color-outline-variant);
      }
      .elevated {
        background: var(--k-color-surface-container-low);
        box-shadow: var(--k-elevation-level1);
      }
      .filled {
        background: var(--k-color-surface-container-highest);
      }
      h2 {
        ${type("title-large")}
        margin: 0;
      }
      .subhead {
        ${type("body-medium")}
        margin: 0;
        color: var(--k-color-on-surface-variant);
      }
      .actions {
        display: flex;
        flex-wrap: wrap;
        justify-content: flex-end;
        gap: var(--k-space-2);
        margin-block-start: auto;
      }
      .actions[hidden] {
        display: none;
      }
    `,
  ];

  /** How it stands out. */
  @property({ reflect: true }) variant: CardVariant = "outlined";

  /** Its heading. */
  @property() heading = "";

  /** A line under the heading. */
  @property() subhead = "";

  /** Whether actions were given. */
  @state() private actions = false;

  override render() {
    return html`<section class=${this.variant} part="base" aria-label=${this.heading || (undefined as never)}>
      ${this.heading ? html`<h2 part="heading">${this.heading}</h2>` : null}
      ${this.subhead ? html`<p class="subhead" part="subhead">${this.subhead}</p>` : null}
      <div part="body"><slot></slot></div>
      <div class="actions" part="actions" ?hidden=${!this.actions}>
        <slot name="actions" @slotchange=${(e: Event) => (this.actions = (e.target as HTMLSlotElement).assignedElements().length > 0)}></slot>
      </div>
    </section>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-card": KCard;
  }
}
