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

import { css, html } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { live } from "lit/directives/live.js";

import { FormControl } from "../../forms/form-control.js";
import { base } from "../../styles/base.js";
import { field } from "../../styles/field.js";
import { stateLayer } from "../../styles/state-layer.js";
import { type } from "../../styles/typography.js";

/** `<k-checkbox label="Remember me" name="remember">`: a labelled check box; it submits `value` when checked. */
@customElement("k-checkbox")
export class KCheckbox extends FormControl {
  static override styles = [
    base,
    field,
    stateLayer,
    css`
      .row {
        display: flex;
        align-items: center;
        gap: var(--k-space-1);
        margin-inline-start: calc(var(--k-space-3) * -1);
      }
      .target {
        display: inline-grid;
        place-items: center;
        inline-size: var(--k-size-control);
        block-size: var(--k-size-control);
        border-radius: var(--k-shape-corner-full);
        color: var(--k-color-primary);
        flex: none;
      }
      input {
        inline-size: 1.125rem;
        block-size: 1.125rem;
        margin: 0;
        accent-color: var(--k-color-primary);
        cursor: pointer;
      }
      input:focus-visible {
        outline-offset: 0.5rem;
        border-radius: var(--k-shape-corner-full);
      }
      label {
        ${type("body-large")}
        cursor: pointer;
      }
      .supporting {
        padding-inline: calc(var(--k-size-control) - var(--k-space-3) + var(--k-space-1));
      }
    `,
  ];

  /** Whether it is checked. */
  @property({ type: Boolean, reflect: true }) checked = false;

  /** What it submits when checked. */
  @property() value = "on";

  /** The native check box. */
  @query("input") private box!: HTMLInputElement | null;

  /** Whether it started checked. */
  private startedChecked = false;

  override connectedCallback(): void {
    super.connectedCallback();
    this.startedChecked = this.checked;
  }

  get formValue(): string | null {
    return this.checked ? this.value : null;
  }

  protected get native(): HTMLInputElement | null {
    return this.box;
  }

  protected restore(): void {
    this.checked = this.startedChecked;
  }

  /** Follows the person. */
  private toggled(event: Event): void {
    this.checked = (event.target as HTMLInputElement).checked;
    this.error = "";
    this.changed();
  }

  override render() {
    return html`<div class="field" part="field">
      <div class="row">
        <span class="target state-layer"><input
          id=${this.uid}
          part="control"
          type="checkbox"
          .checked=${live(this.checked)}
          ?required=${this.required}
          ?disabled=${this.disabled}
          aria-invalid=${this.error ? "true" : "false"}
          aria-describedby=${this.describedBy()}
          @change=${this.toggled}
        /></span>
        <label for=${this.uid} part="label">${this.label}</label>
      </div>
      ${this.supporting()}
    </div>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-checkbox": KCheckbox;
  }
}
