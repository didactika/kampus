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
// @package   @kampus/widget-text
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


/** A heading and a text, as the scope wrote them. */

import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";

import { text, type WidgetProps } from "@kampus/web/host";

/** A heading and a text. */
export function TextWidget({ settings }: WidgetProps) {
  const heading = text(settings, "heading");
  const body = text(settings, "text");
  if (!heading && !body) return null;
  return (
    <Card>
      <CardContent sx={{ p: 3 }}>
        {heading ? (
          <Typography variant="h3" component="h2" sx={{ mb: 1 }}>
            {heading}
          </Typography>
        ) : null}
        {body ? (
          <Typography color="text.secondary" sx={{ whiteSpace: "pre-line" }}>
            {body}
          </Typography>
        ) : null}
      </CardContent>
    </Card>
  );
}
