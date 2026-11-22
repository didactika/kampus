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

/**
 * What the controls of a form share: the label above the control, the outlined box with its focus and error, and the
 * supporting text under it. The label stays above rather than inside, so it is always read and never covers a value.
 */

import { css } from "lit";

import { type } from "./typography.js";

export const field = css`
  :host {
    display: block;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--k-space-1);
  }
  label {
    ${type("label-large")}
    color: var(--k-color-on-surface);
  }
  .required {
    color: var(--k-color-danger);
    margin-inline-start: var(--k-space-1);
  }
  .supporting {
    ${type("body-small")}
    margin: 0;
    padding-inline: var(--k-space-4);
    color: var(--k-color-on-surface-variant);
  }
  .supporting.error {
    color: var(--k-color-danger);
  }
  .control {
    ${type("body-large")}
    color: var(--k-color-on-surface);
    background: transparent;
    border: 1px solid var(--k-color-outline);
    border-radius: var(--k-shape-corner-small);
    padding-block: var(--k-space-2);
    padding-inline: var(--k-space-4);
    min-block-size: var(--k-size-field);
    inline-size: 100%;
    transition:
      border-color var(--k-duration-short3) var(--k-easing-standard),
      box-shadow var(--k-duration-short3) var(--k-easing-standard);
  }
  .control:hover:not(:disabled) {
    border-color: var(--k-color-on-surface);
  }
  .control:focus-visible {
    outline: none;
    border-color: var(--k-color-primary);
    box-shadow: inset 0 0 0 1px var(--k-color-primary);
  }
  .control[aria-invalid="true"] {
    border-color: var(--k-color-danger);
  }
  .control[aria-invalid="true"]:focus-visible {
    box-shadow: inset 0 0 0 1px var(--k-color-danger);
  }
  .control::placeholder {
    color: var(--k-color-on-surface-variant);
  }
  .control:disabled {
    cursor: not-allowed;
    color: color-mix(in srgb, var(--k-color-on-surface) calc(var(--k-state-disabled-content) * 100%), transparent);
    border-color: color-mix(in srgb, var(--k-color-on-surface) calc(var(--k-state-disabled-container) * 100%), transparent);
  }
`;
