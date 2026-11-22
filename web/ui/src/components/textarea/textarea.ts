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

import { css, html, nothing } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { live } from "lit/directives/live.js";

import { FormControl } from "../../forms/form-control.js";
import { base } from "../../styles/base.js";
import { field } from "../../styles/field.js";

/** `<k-textarea label="Description" name="description" rows="4">`: a labelled field of several lines. */
@customElement("k-textarea")
export class KTextarea extends FormControl {
  static override styles = [
    base,
    field,
    css`
      textarea {
        resize: vertical;
        min-block-size: calc(var(--k-size-field) * 2);
      }
    `,
  ];

  /** The value. */
  @property() value = "";

  /** How many lines show. */
  @property({ type: Number }) rows = 4;

  /** The longest text. */
  @property({ type: Number }) maxlength?: number;

  /** The native text area. */
  @query("textarea") private area!: HTMLTextAreaElement | null;

  override connectedCallback(): void {
    super.connectedCallback();
    this.initial = this.value;
  }

  get formValue(): string | null {
    return this.value;
  }

  protected get native(): HTMLTextAreaElement | null {
    return this.area;
  }

  protected restore(value: string): void {
    this.value = value;
  }

  /** Follows what the person types. */
  private typed(event: Event): void {
    this.value = (event.target as HTMLTextAreaElement).value;
    this.error = "";
    this.sync();
  }

  override render() {
    return this.frame(html`<textarea
      class="control"
      part="control"
      id=${this.uid}
      rows=${this.rows}
      .value=${live(this.value)}
      maxlength=${this.maxlength ?? nothing}
      ?required=${this.required}
      ?disabled=${this.disabled}
      aria-invalid=${this.error ? "true" : "false"}
      aria-describedby=${this.describedBy()}
      @input=${this.typed}
      @change=${this.changed}
    ></textarea>`);
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-textarea": KTextarea;
  }
}
