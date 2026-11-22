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

"use client";

/** A course in a list: its name opens it, with its short name and the start of its summary. */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardActionArea from "@mui/material/CardActionArea";
import CardContent from "@mui/material/CardContent";
import Typography from "@mui/material/Typography";
import Link from "next/link";

import { Icon } from "@kampus/ui";

import type { Course } from "../../api/types";

/** `<CourseCard course={…} />` */
export function CourseCard({ course }: { readonly course: Course }) {
  return (
    <Card sx={{ blockSize: "100%", transition: "box-shadow 200ms", "&:hover": { boxShadow: 2 } }}>
      <CardActionArea component={Link} href={`/courses/${course.id}`} sx={{ blockSize: "100%", alignItems: "stretch" }}>
        <Box
          aria-hidden
          sx={{
            blockSize: 88,
            display: "flex",
            alignItems: "flex-end",
            p: 2,
            color: "primary.onContainer",
            background: "linear-gradient(135deg, var(--k-palette-primary-container), var(--k-palette-accent-container))",
          }}
        >
          <Icon name="school" size="large" />
        </Box>
        <CardContent>
          <Typography variant="overline" color="text.secondary">
            {course.short_name}
          </Typography>
          <Typography variant="h4" component="h3" sx={{ mt: 0.5 }}>
            {course.full_name}
          </Typography>
          {course.summary ? (
            <Typography
              variant="body2"
              color="text.secondary"
              sx={{ mt: 1, display: "-webkit-box", WebkitLineClamp: 3, WebkitBoxOrient: "vertical", overflow: "hidden" }}
            >
              {course.summary}
            </Typography>
          ) : null}
        </CardContent>
      </CardActionArea>
    </Card>
  );
}
