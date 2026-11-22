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

import { FormControl } from "../../forms/form-control.js";
import { base } from "../../styles/base.js";
import { field } from "../../styles/field.js";

/** One option of a `k-select`. */
export interface SelectOption {
  /** What it submits. */
  readonly value: string;
  /** What it shows. */
  readonly label: string;
  /** Whether it cannot be chosen. */
  readonly disabled?: boolean;
}

/** `<k-select label="Language" name="language" .options=${[...]}>`: a labelled native list to choose one from. */
@customElement("k-select")
export class KSelect extends FormControl {
  static override styles = [
    base,
    field,
    css`
      .box {
        position: relative;
      }
      select {
        appearance: none;
        padding-inline-end: var(--k-space-10);
        cursor: pointer;
      }
      .box::after {
        content: "";
        position: absolute;
        inset-inline-end: var(--k-space-4);
        inset-block-start: 50%;
        inline-size: 0.5rem;
        block-size: 0.5rem;
        border: solid var(--k-color-on-surface-variant);
        border-width: 0 2px 2px 0;
        transform: translateY(-75%) rotate(45deg);
        pointer-events: none;
      }
    `,
  ];

  /** The options. */
  @property({ attribute: false }) options: readonly SelectOption[] = [];

  /** The value chosen. */
  @property() value = "";

  /** The native list. */
  @query("select") private list!: HTMLSelectElement | null;

  override connectedCallback(): void {
    super.connectedCallback();
    this.initial = this.value;
  }

  get formValue(): string | null {
    return this.value;
  }

  protected get native(): HTMLSelectElement | null {
    return this.list;
  }

  protected restore(value: string): void {
    this.value = value;
  }

  /** Follows the person. */
  private chose(event: Event): void {
    this.value = (event.target as HTMLSelectElement).value;
    this.error = "";
    this.changed();
  }

  override render() {
    return this.frame(html`<div class="box"><select
      class="control"
      part="control"
      id=${this.uid}
      ?required=${this.required}
      ?disabled=${this.disabled}
      aria-invalid=${this.error ? "true" : "false"}
      aria-describedby=${this.describedBy()}
      @change=${this.chose}
    >
      ${this.options.map(
        (option) =>
          html`<option value=${option.value} ?selected=${option.value === this.value} ?disabled=${option.disabled ?? false}>
            ${option.label}
          </option>`,
      )}
    </select></div>`);
  }

  override updated(): void {
    if (this.list && this.list.value !== this.value) this.list.value = this.value;
    super.updated();
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-select": KSelect;
  }
}

