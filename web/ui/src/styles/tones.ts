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
 * The colour roles of a tone as four local properties (`--tone`, `--on-tone`, `--tone-container`,
 * `--on-tone-container`), chosen by the `tone` attribute of the host, so a component styles every tone at once.
 */

import { css, unsafeCSS } from "lit";

/** The tones a component can take. */
const TONES = ["primary", "secondary", "accent", "info", "success", "warning", "danger"] as const;

/** The declarations of one tone. */
function tone(name: string): string {
  return (
    `--tone: var(--k-color-${name}); --on-tone: var(--k-color-on-${name}); ` +
    `--tone-container: var(--k-color-${name}-container); --on-tone-container: var(--k-color-on-${name}-container);`
  );
}

export const tones = css`
  :host {
    ${unsafeCSS(tone("secondary"))}
  }
  ${unsafeCSS(TONES.map((name) => `:host([tone="${name}"]) { ${tone(name)} }`).join("\n"))}
`;
