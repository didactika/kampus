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

/** The look of `k-button`: a pill as tall as a control, its variant and tone from the colour roles. */

import { css } from "lit";

import { type } from "../../styles/typography.js";

export const buttonStyles = css`
  :host {
    display: inline-block;
    --tone: var(--k-color-primary);
    --on-tone: var(--k-color-on-primary);
    --tone-container: var(--k-color-primary-container);
    --on-tone-container: var(--k-color-on-primary-container);
  }
  :host([tone="danger"]) {
    --tone: var(--k-color-danger);
    --on-tone: var(--k-color-on-danger);
    --tone-container: var(--k-color-danger-container);
    --on-tone-container: var(--k-color-on-danger-container);
  }
  :host([tone="success"]) {
    --tone: var(--k-color-success);
    --on-tone: var(--k-color-on-success);
    --tone-container: var(--k-color-success-container);
    --on-tone-container: var(--k-color-on-success-container);
  }
  button {
    ${type("label-large")}
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--k-space-2);
    inline-size: 100%;
    min-block-size: var(--k-size-control);
    min-inline-size: var(--k-size-control);
    padding-block: 0;
    padding-inline: var(--k-space-6);
    border-radius: var(--k-shape-corner-full);
    border: 1px solid transparent;
    cursor: pointer;
    user-select: none;
    transition:
      box-shadow var(--k-duration-short4) var(--k-easing-standard),
      background-color var(--k-duration-short4) var(--k-easing-standard);
  }
  .filled {
    background: var(--tone);
    color: var(--on-tone);
  }
  .filled:hover:not(:disabled, [aria-disabled="true"]) {
    box-shadow: var(--k-elevation-level1);
  }
  .tonal {
    background: var(--tone-container);
    color: var(--on-tone-container);
  }
  .outlined {
    background: transparent;
    color: var(--tone);
    border-color: var(--k-color-outline);
  }
  .text {
    background: transparent;
    color: var(--tone);
    padding-inline: var(--k-space-3);
  }
  .elevated {
    background: var(--k-color-surface-container-low);
    color: var(--tone);
    box-shadow: var(--k-elevation-level1);
  }
  .elevated:hover:not(:disabled, [aria-disabled="true"]) {
    box-shadow: var(--k-elevation-level2);
  }
  button:disabled {
    cursor: not-allowed;
    box-shadow: none;
    color: color-mix(in srgb, var(--k-color-on-surface) calc(var(--k-state-disabled-content) * 100%), transparent);
    background: color-mix(in srgb, var(--k-color-on-surface) calc(var(--k-state-disabled-container) * 100%), transparent);
    border-color: transparent;
  }
  .text:disabled,
  .outlined:disabled {
    background: transparent;
  }
  .outlined:disabled {
    border-color: color-mix(in srgb, var(--k-color-on-surface) calc(var(--k-state-disabled-container) * 100%), transparent);
  }
  button[aria-busy="true"] {
    cursor: progress;
  }
  ::slotted([slot="icon"]) {
    inline-size: 1.125rem;
    block-size: 1.125rem;
    margin-inline-start: calc(var(--k-space-2) * -1);
  }
  .spinner {
    inline-size: 1.125rem;
    block-size: 1.125rem;
    border: 2px solid currentColor;
    border-inline-end-color: transparent;
    border-radius: 50%;
    animation: spin calc(800ms * var(--k-motion) + 1ms) linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
`;
