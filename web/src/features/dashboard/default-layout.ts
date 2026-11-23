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
// @package   web
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


/** The home a campus shows until it arranges its own: the welcome across the top, the catalogue, and the inbox beside it. */

import type { Placement } from "../../api/types";

/** The grid of a page until the server says: twelve columns of regular rows. */
export const STANDARD_GRID = { columns: 12, row: 72 } as const;

/** The page kind of the home. */
export const DASHBOARD = "customise.dashboard";

/** The widgets of the home by default: the welcome and the latest notifications side by side, the catalogue under them. */
export const DEFAULT_DASHBOARD: readonly Placement[] = [
  { widget: "widget_welcome", x: 0, y: 0, w: 8, h: 5, settings: { heading: "" }, pinned: false, locked: false, held: false, grows: false, permission: null },
  { widget: "widget_notifications", x: 8, y: 0, w: 4, h: 5, settings: { limit: 5 }, pinned: false, locked: false, held: false, grows: false, permission: null },
  { widget: "widget_catalogue", x: 0, y: 5, w: 12, h: 8, settings: { heading: "", view: "grid", size: 8 }, pinned: false, locked: false, held: false, grows: false, permission: null },
];
