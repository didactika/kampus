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
 * The widgets the shell knows how to show, by the code their module declares. Each is a server component that reads
 * what it needs for the person of the request; a widget with no entry here is skipped, so a page never breaks.
 */

import type { ReactNode } from "react";

import { CatalogueWidget } from "./CatalogueWidget";
import { InboxWidget } from "./InboxWidget";
import { TextWidget } from "./TextWidget";
import { WelcomeWidget } from "./WelcomeWidget";

/** What a widget receives: its settings, with the defaults its module declares, and the campus of the page. */
export interface WidgetProps {
  readonly settings: Readonly<Record<string, unknown>>;
  readonly campus: string;
}

/** A widget: a server component. */
export type Widget = (props: WidgetProps) => Promise<ReactNode> | ReactNode;

/** The widgets, by code. */
export const WIDGETS: Readonly<Record<string, Widget>> = {
  "customise.welcome": WelcomeWidget,
  "customise.text": TextWidget,
  "courses.catalogue": CatalogueWidget,
  "notify.inbox": InboxWidget,
};

/** A setting of a widget as text. */
export function text(settings: WidgetProps["settings"], key: string): string {
  const value = settings[key];
  return typeof value === "string" ? value : "";
}

/** A setting of a widget as a number. */
export function number(settings: WidgetProps["settings"], key: string, fallback: number): number {
  const value = settings[key];
  return typeof value === "number" ? value : fallback;
}
