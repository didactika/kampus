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
 * The state layer: a veil of the colour of the content over an element that can be pressed, more opaque on hover,
 * on keyboard focus and while pressed, as the design system says how an element answers. An element opts in with
 * the class `state-layer`.
 */

import { css } from "lit";

export const stateLayer = css`
  .state-layer {
    position: relative;
    isolation: isolate;
    overflow: hidden;
  }
  .state-layer::before {
    content: "";
    position: absolute;
    inset: 0;
    z-index: -1;
    border-radius: inherit;
    background: currentColor;
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--k-duration-short2) var(--k-easing-standard);
  }
  .state-layer:hover:not(:disabled, [aria-disabled="true"])::before {
    opacity: var(--k-state-hover);
  }
  .state-layer:focus-visible::before {
    opacity: var(--k-state-focus);
  }
  .state-layer:active:not(:disabled, [aria-disabled="true"])::before {
    opacity: var(--k-state-pressed);
  }
`;
