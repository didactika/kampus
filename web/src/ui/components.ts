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
// @package   web/src
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

/** `<k-field label="…" name="…" type="…">`: a labelled input; the label, the error and the input are tied together. */
export class KField extends HTMLElement {
  private static counter = 0;
  private input: HTMLInputElement | null = null;

  connectedCallback(): void {
    if (this.input) return;
    const id = `k-field-${(KField.counter += 1)}`;
    const label = document.createElement("label");
    label.htmlFor = id;
    label.textContent = this.getAttribute("label") ?? "";
    const input = document.createElement("input");
    input.id = id;
    input.name = this.getAttribute("name") ?? "";
    input.type = this.getAttribute("type") ?? "text";
    input.setAttribute("autocomplete", this.getAttribute("autocomplete") ?? "off");
    input.required = this.hasAttribute("required");
    this.input = input;
    this.replaceChildren(label, input);
  }

  /** What the person typed. */
  get value(): string {
    return this.input?.value ?? "";
  }

  /** Puts the focus in the input. */
  override focus(): void {
    this.input?.focus();
  }
}

/** `<k-button variant="quiet">`: a button; its text is the text of the element. */
export class KButton extends HTMLElement {
  private button: HTMLButtonElement | null = null;

  connectedCallback(): void {
    if (this.button) return;
    const button = document.createElement("button");
    button.type = this.getAttribute("type") === "submit" ? "submit" : "button";
    button.textContent = this.textContent;
    button.disabled = this.hasAttribute("disabled");
    this.button = button;
    this.replaceChildren(button);
  }

  static get observedAttributes(): string[] {
    return ["disabled"];
  }

  attributeChangedCallback(): void {
    if (this.button) this.button.disabled = this.hasAttribute("disabled");
  }
}

/** Registers the components of the design system. */
export function defineComponents(): void {
  if (!customElements.get("k-field")) customElements.define("k-field", KField);
  if (!customElements.get("k-button")) customElements.define("k-button", KButton);
}
