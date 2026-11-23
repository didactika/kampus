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

/**
 * Rich text as the reader sees it: HTML the server sanitised when it kept it and again when it gave it, shown with the
 * typography of the page. It is never given HTML from anywhere else.
 */

import Box, { type BoxProps } from "@mui/material/Box";

/** The look of rich text, the same when it is read and while it is edited. */
export const RICH_TEXT_LOOK = {
  "& p": { my: 0.5 },
  "& p:first-of-type": { mt: 0 },
  "& a": { color: "primary.main" },
  "& ul, & ol": { my: 0.5, pl: 3 },
  "& blockquote": { borderInlineStart: 3, borderColor: "divider", pl: 1.5, ml: 0, color: "text.secondary" },
  "& code": { fontFamily: "monospace", fontSize: "0.9em", px: 0.5, borderRadius: 1, bgcolor: "action.hover" },
} as const;

/** The props of `RichText`. */
export interface RichTextProps extends Omit<BoxProps, "children" | "dangerouslySetInnerHTML"> {
  /** HTML in the format of user content, as the server gave it. */
  readonly html: string;
}

/** `<RichText html={settings.text} />` */
export function RichText({ html, sx, ...rest }: RichTextProps) {
  return (
    <Box
      {...rest}
      sx={[RICH_TEXT_LOOK, ...(Array.isArray(sx) ? sx : [sx])]}
      // The server sanitised this HTML in the one format of user content; nothing in it runs.
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
