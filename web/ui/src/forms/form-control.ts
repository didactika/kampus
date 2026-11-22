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

import { LitElement, html, nothing, type TemplateResult } from "lit";
import { property } from "lit/decorators.js";

let counter = 0;

/**
 * A control that takes part in the form around it, whatever framework renders it: its value is submitted with the
 * form, it is reset and disabled with it, and the browser validates it. Its label, hint and error are tied to the
 * native control inside it, so assistive technology reads them together.
 */
export abstract class FormControl extends LitElement {
  /** The controls take part in forms. */
  static readonly formAssociated = true;

  /** Focus goes to the native control inside. */
  static override shadowRootOptions = { ...LitElement.shadowRootOptions, delegatesFocus: true };

  /** The link to the form. */
  protected readonly internals: ElementInternals;

  /** An id of its own, for the label, hint and error inside it. */
  protected readonly uid = `k-${++counter}`;

  /** The name the value is submitted under. */
  @property({ reflect: true }) name = "";

  /** The visible label; every control has one. */
  @property() label = "";

  /** Help shown under the control. */
  @property() hint = "";

  /** An error to show, such as one the server returned; the browser's own validity is shown otherwise. */
  @property() error = "";

  /** Whether a value is required. */
  @property({ type: Boolean, reflect: true }) required = false;

  /** Whether it cannot be changed. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** The value it starts with, restored when the form is reset. */
  protected initial = "";

  constructor() {
    super();
    this.internals = this.attachInternals();
  }

  /** The value submitted with the form. */
  abstract get formValue(): string | null;

  /** The native control inside, for validity and focus. */
  protected abstract get native(): HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement | null;

  /** Puts the value back to the one it started with. */
  protected abstract restore(value: string): void;

  /** The form this control belongs to. */
  get form(): HTMLFormElement | null {
    return this.internals.form;
  }

  /** Whether the value is valid, showing the reason when it is not. */
  reportValidity(): boolean {
    this.sync();
    return this.internals.reportValidity();
  }

  /** Whether the value is valid. */
  checkValidity(): boolean {
    this.sync();
    return this.internals.checkValidity();
  }

  /** Tells the form the value and the validity of the control. */
  protected sync(): void {
    this.internals.setFormValue(this.formValue);
    const native = this.native;
    if (this.error) {
      this.internals.setValidity({ customError: true }, this.error, native ?? undefined);
    } else if (native && !native.validity.valid) {
      this.internals.setValidity(native.validity, native.validationMessage, native);
    } else {
      this.internals.setValidity({});
    }
  }

  /** The form was reset. */
  formResetCallback(): void {
    this.restore(this.initial);
    this.error = "";
    this.sync();
  }

  /** The form, or a fieldset around the control, was disabled or enabled. */
  formDisabledCallback(disabled: boolean): void {
    this.disabled = disabled;
  }

  /** Re-announces a change from inside, which does not cross the shadow root by itself. */
  protected changed(): void {
    this.sync();
    this.dispatchEvent(new Event("change", { bubbles: true }));
  }

  override updated(): void {
    this.sync();
  }

  /** The ids the native control is described by. */
  protected describedBy(): string | typeof nothing {
    const ids = [this.hint ? `${this.uid}-hint` : "", this.error ? `${this.uid}-error` : ""].filter(Boolean);
    return ids.length ? ids.join(" ") : nothing;
  }

  /** The hint and the error under the control. */
  protected supporting(): TemplateResult {
    return html`${this.hint ? html`<p class="supporting" id="${this.uid}-hint" part="hint">${this.hint}</p>` : nothing}
    ${this.error ? html`<p class="supporting error" id="${this.uid}-error" part="error">${this.error}</p>` : nothing}`;
  }

  /** The label, the control, the hint and the error, in that order. */
  protected frame(control: TemplateResult): TemplateResult {
    return html`<div class="field" part="field">
      <label for=${this.uid} part="label"
        >${this.label}${this.required ? html`<span class="required" aria-hidden="true">*</span>` : nothing}</label
      >
      ${control}${this.supporting()}
    </div>`;
  }
}
