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
import { customElement, property, queryAssignedElements, state } from "lit/decorators.js";

import { base } from "../../styles/base.js";
import { stateLayer } from "../../styles/state-layer.js";
import { type } from "../../styles/typography.js";

let panels = 0;

/** `<k-tab-panel label="Grades">…</k-tab-panel>`: one panel of a `k-tabs`. */
@customElement("k-tab-panel")
export class KTabPanel extends LitElement {
  static override styles = css`
    :host {
      display: block;
    }
    :host([hidden]) {
      display: none;
    }
  `;

  /** The label of its tab. */
  @property() label = "";

  override connectedCallback(): void {
    super.connectedCallback();
    this.setAttribute("role", "tabpanel");
    if (!this.hasAttribute("tabindex")) this.tabIndex = 0;
  }

  override render() {
    return html`<slot></slot>`;
  }
}

/**
 * `<k-tabs label="Course"><k-tab-panel label="…">…</k-tab-panel>…</k-tabs>`: tabs with the keyboard pattern of
 * WAI-ARIA: arrows move between tabs, Home and End go to the first and last, and only the selected tab is in the tab
 * order. The panels stay in the light DOM, so their content belongs to the page.
 */
@customElement("k-tabs")
export class KTabs extends LitElement {
  static override styles = [
    base,
    stateLayer,
    css`
      :host {
        display: block;
      }
      [role="tablist"] {
        display: flex;
        border-block-end: 1px solid var(--k-color-outline-variant);
        margin-block-end: var(--k-space-4);
        overflow-x: auto;
        scrollbar-width: none;
      }
      button {
        ${type("title-small")}
        position: relative;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-block-size: calc(var(--k-size-control) + var(--k-space-2));
        padding-inline: var(--k-space-4);
        background: transparent;
        color: var(--k-color-on-surface-variant);
        border: none;
        cursor: pointer;
        white-space: nowrap;
        flex: 1 0 auto;
      }
      button[aria-selected="true"] {
        color: var(--k-color-primary);
      }
      button[aria-selected="true"]::after {
        content: "";
        position: absolute;
        inset-inline: var(--k-space-4);
        inset-block-end: 0;
        block-size: 3px;
        background: var(--k-color-primary);
        border-start-start-radius: 3px;
        border-start-end-radius: 3px;
      }
      button:focus-visible {
        outline-offset: -3px;
      }
    `,
  ];

  /** The name of the set of tabs. */
  @property() label = "";

  /** The index of the selected tab. */
  @property({ type: Number, reflect: true }) selected = 0;

  /** The panels. */
  @queryAssignedElements({ selector: "k-tab-panel" }) private panels!: KTabPanel[];

  /** The labels of the panels, read when they change. */
  @state() private labels: string[] = [];

  /** The tabs rendered. */
  private tabs(): HTMLButtonElement[] {
    return [...(this.renderRoot.querySelectorAll<HTMLButtonElement>("[role=tab]") ?? [])];
  }

  /** Reads the panels. */
  private slotted(): void {
    this.labels = this.panels.map((p) => p.label);
    this.show();
  }

  /** Shows the selected panel and ties each panel to its tab. */
  private show(): void {
    this.panels.forEach((panel, index) => {
      panel.hidden = index !== this.selected;
      if (!panel.id) panel.id = `k-tab-panel-${++panels}`;
      panel.setAttribute("aria-label", panel.label);
    });
  }

  /** Selects a tab and moves focus to it. */
  select(index: number, focus = false): void {
    const count = this.labels.length;
    if (count === 0) return;
    this.selected = ((index % count) + count) % count;
    this.dispatchEvent(new CustomEvent("k-tab-change", { bubbles: true, composed: true, detail: { index: this.selected } }));
    if (focus) void this.updateComplete.then(() => this.tabs()[this.selected]?.focus());
  }

  /** The keyboard pattern of tabs. */
  private keyed(event: KeyboardEvent): void {
    const moves: Record<string, number> = {
      ArrowRight: this.selected + 1,
      ArrowLeft: this.selected - 1,
      Home: 0,
      End: this.labels.length - 1,
    };
    const rtl = getComputedStyle(this).direction === "rtl";
    if (rtl) [moves["ArrowRight"], moves["ArrowLeft"]] = [moves["ArrowLeft"]!, moves["ArrowRight"]!];
    const target = moves[event.key];
    if (target === undefined) return;
    event.preventDefault();
    this.select(target, true);
  }

  override updated(): void {
    this.show();
  }

  override render() {
    return html`<div role="tablist" aria-label=${this.label} part="tablist" @keydown=${this.keyed}>
        ${this.labels.map(
          (label, index) =>
            html`<button
              role="tab"
              part="tab"
              class="state-layer"
              type="button"
              aria-selected=${index === this.selected ? "true" : "false"}
              tabindex=${index === this.selected ? 0 : -1}
              @click=${() => this.select(index)}
            >
              ${label}
            </button>`,
        )}
      </div>
      <slot @slotchange=${this.slotted}></slot>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-tabs": KTabs;
    "k-tab-panel": KTabPanel;
  }
}
