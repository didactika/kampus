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

import { LitElement, html } from "lit";
import { customElement, property } from "lit/decorators.js";

import { buttonStyles } from "./button.styles.js";
import { base } from "../../styles/base.js";
import { stateLayer } from "../../styles/state-layer.js";

/**
 * How a button looks, from the most emphasis to the least: `filled` for the main action of an area (one per area),
 * `tonal` and `elevated` for important ones beside it, `outlined` for ordinary actions, `text` for the least.
 */
export type ButtonVariant = "filled" | "tonal" | "elevated" | "outlined" | "text";

/** What a button does: an ordinary action, one that removes or stops (it always asks first), or one that starts. */
export type ButtonTone = "primary" | "danger" | "success";

/**
 * `<k-button variant="filled" type="submit">Save</k-button>`: a button that submits or resets the form it is in,
 * and says when it is busy so a second press does nothing.
 */
@customElement("k-button")
export class KButton extends LitElement {
  /** Buttons take part in forms. */
  static readonly formAssociated = true;

  /** Focus goes to the native button. */
  static override shadowRootOptions = { ...LitElement.shadowRootOptions, delegatesFocus: true };

  static override styles = [base, stateLayer, buttonStyles];

  /** The look. */
  @property({ reflect: true }) variant: ButtonVariant = "outlined";

  /** What it does. */
  @property({ reflect: true }) tone: ButtonTone = "primary";

  /** What it does in a form. */
  @property({ reflect: true }) type: "button" | "submit" | "reset" = "button";

  /** Whether it cannot be pressed. */
  @property({ type: Boolean, reflect: true }) disabled = false;

  /** Whether the action it started is still running; it cannot be pressed meanwhile. */
  @property({ type: Boolean, reflect: true }) busy = false;

  /** A name for assistive technology when the button shows only an icon. */
  @property() label = "";

  /** The link to the form. */
  private readonly internals: ElementInternals;

  constructor() {
    super();
    this.internals = this.attachInternals();
  }

  /** Submits or resets the form, as a native button would. */
  private pressed(event: MouseEvent): void {
    if (this.disabled || this.busy) {
      event.preventDefault();
      event.stopImmediatePropagation();
      return;
    }
    const form = this.internals.form;
    if (!form) return;
    if (this.type === "submit") form.requestSubmit();
    if (this.type === "reset") form.reset();
  }

  override render() {
    return html`<button
      part="base"
      class="state-layer ${this.variant}"
      type="button"
      ?disabled=${this.disabled}
      aria-disabled=${this.busy ? "true" : "false"}
      aria-busy=${this.busy ? "true" : "false"}
      aria-label=${this.label || (undefined as never)}
      @click=${this.pressed}
    >
      ${this.busy ? html`<span class="spinner" aria-hidden="true"></span>` : html`<slot name="icon"></slot>`}<slot></slot>
    </button>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "k-button": KButton;
  }
}
