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


/** What every widget receives, and the helpers that read its settings. */

import type { ReactNode } from "react";

import type { CourseData } from "../features/course/course-data";

/** The page a widget is placed on: its campus and, on the page of a course, what the page read of the course. */
export interface PageContext {
  readonly campus: string;
  readonly course?: CourseData;
}

/** The size a widget has on the grid of its page, in cells: columns of twelve across the page, rows of their height. */
export interface WidgetSize {
  readonly w: number;
  readonly h: number;
}

/**
 * What a widget receives: its settings, with the defaults its module declares, the page it is on, and the size it
 * has there, from which it chooses how to show its content so that it never spills out of it.
 */
export interface WidgetProps {
  readonly settings: Readonly<Record<string, unknown>>;
  readonly context: PageContext;
  readonly size: WidgetSize;
  /** The language of the reader, in which a text written per language is read. */
  readonly language: string;
}

/** A widget: a server component. */
export type Widget = (props: WidgetProps) => Promise<ReactNode> | ReactNode;

/** A widget as its plugin gives it to the web client: the code of its declaration and how it renders. */
export interface WidgetPlugin {
  /** The name of the plugin, `widget_<name>`, which is the code of the widget. */
  readonly code: string;
  readonly render: Widget;
}

/** The pixels of a row of the grid and of the room between rows, for a widget to work out what fits in its height. */
const ROW = 72;
const GAP = 24;

/** The height of a widget, in pixels, from its rows. */
export function heightOf(size: WidgetSize): number {
  return size.h * ROW + (size.h - 1) * GAP;
}

/** How many things of a height fit in a widget once its own header and footer are taken out; at least one. */
export function fitting(size: WidgetSize, chrome: number, each: number): number {
  return Math.max(1, Math.floor((heightOf(size) - chrome) / each));
}

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
