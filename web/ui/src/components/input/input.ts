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

import { html, nothing } from "lit";
import { customElement, property, query } from "lit/decorators.js";
import { live } from "lit/directives/live.js";

import { FormControl } from "../../forms/form-control.js";
import { base } from "../../styles/base.js";
import { field } from "../../styles/field.js";

/** The kinds of text a `k-input` takes. */
export type InputType = "text" | "email" | "password" | "search" | "url" | "tel" | "number" | "date" | "time" | "datetime-local";

/** `<k-input label="Email" name="email" type="email" required>`: a labelled text field that takes part in forms. */
@customElement("k-input")
export class KInput extends FormControl {
  static override styles = [base, field];

  /** The kind of text. */
  @property() type: InputType = "text";

  /** The value. */
  @property() value = "";

  /** A hint inside the empty field; never a replacement for the label. */
  @property() placeholder = "";

  /** What the browser may fill in, such as `email` or `current-password`. */
  @property() autocomplete = "";

  /** The shortest and longest text. */
  @property({ type: Number }) minlength?: number;
  @property({ type: Number }) maxlength?: number;

  /** The pattern the text must match. */
  @property() pattern = "";

  /** The smallest and largest number or date, and the step between them. */
  @property() min = "";
  @property() max = "";
  @property() step = "";

  /** The native input. */
  @query("input") private input!: HTMLInputElement | null;

  override connectedCallback(): void {
    super.connectedCallback();
    this.initial = this.value;
  }

  get formValue(): string | null {
    return this.value;
  }

  protected get native(): HTMLInputElement | null {
    return this.input;
  }

  protected restore(value: string): void {
    this.value = value;
  }

  /** Follows what the person types. */
  private typed(event: Event): void {
    this.value = (event.target as HTMLInputElement).value;
    this.error = "";
    this.sync();
  }

  override render() {
    return this.frame(html`<input
      class="control"
      part="control"
      id=${this.uid}
      .type=${this.type}
      .value=${live(this.value)}
      name=${this.name || nothing}
      placeholder=${this.placeholder || nothing}
      autocomplete=${(this.autocomplete || nothing) as never}
      minlength=${this.minlength ?? nothing}
      maxlength=${this.maxlength ?? nothing}
      pattern=${this.pattern || nothing}
      min=${this.min || nothing}
      max=${this.max || nothing}
      step=${this.step || nothing}
      ?required=${this.required}
      ?disabled=${this.disabled}
      aria-invalid=${this.error ? "true" : "false"}
      aria-describedby=${this.describedBy()}
      @input=${this.typed}
      @change=${this.changed}
    />`);
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-input": KInput;
  }
}
