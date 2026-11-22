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

/**
 * One notification: the icon of its type in its colour, what it says, how long ago, and a dot while it is unread.
 * When it is about a course it opens the course (and is marked read); an unread one can be marked read.
 */

import Box from "@mui/material/Box";
import ButtonBase from "@mui/material/ButtonBase";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import Link from "next/link";

import { Icon } from "@kampus/ui";

import type { Notification } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { describe } from "./describe";
import { courseOf, lookOf } from "./look";

/** The props of `NotificationRow`. */
export interface NotificationRowProps {
  readonly notification: Notification;
  /** Marks it read. */
  readonly onRead?: (id: string) => void;
  /** Smaller, for a widget. */
  readonly dense?: boolean;
}

/** `<NotificationRow notification={…} onRead={…} />` */
export function NotificationRow({ notification: n, onRead, dense = false }: NotificationRowProps) {
  const { t, relative, date } = useI18n();
  const look = lookOf(n.type);
  const course = courseOf(n.payload);
  const unread = !n.read_at;
  const content = (
    <Stack direction="row" spacing={dense ? 1.5 : 2} sx={{ alignItems: "center", flex: 1, minInlineSize: 0, p: dense ? 1.25 : 1.75, textAlign: "start" }}>
      <Box
        sx={{
          display: "grid",
          placeItems: "center",
          inlineSize: dense ? 36 : 44,
          blockSize: dense ? 36 : 44,
          flex: "none",
          borderRadius: 2.5,
          bgcolor: `${look.tone}.container`,
          color: `${look.tone}.onContainer`,
        }}
      >
        <Icon name={look.icon} size={dense ? "small" : "medium"} />
      </Box>
      <Box sx={{ flex: 1, minInlineSize: 0 }}>
        <Typography variant={dense ? "subtitle2" : "subtitle1"} noWrap={dense} sx={{ fontWeight: unread ? 700 : 500 }}>
          {describe(t, n.type)}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          <time dateTime={n.created_at} title={date(n.created_at)}>
            {relative(n.created_at)}
          </time>
          {course && !dense ? ` · ${t("notifications.open")}` : null}
        </Typography>
      </Box>
      {unread ? <Box aria-hidden sx={{ inlineSize: 10, blockSize: 10, borderRadius: "50%", flex: "none", backgroundImage: "var(--k-palette-gradient-primary)" }} /> : null}
    </Stack>
  );
  return (
    <Stack
      direction="row"
      sx={{
        alignItems: "center",
        borderRadius: 3,
        bgcolor: unread ? "color-mix(in srgb, var(--k-palette-primary-main) 6%, transparent)" : "transparent",
        transition: "background-color 140ms",
        "&:hover": { bgcolor: "color-mix(in srgb, var(--k-palette-primary-main) 10%, transparent)" },
      }}
    >
      {course ? (
        <ButtonBase component={Link} href={`/courses/${course}`} onClick={() => unread && onRead?.(n.id)} sx={{ flex: 1, minInlineSize: 0, borderRadius: 3, justifyContent: "flex-start" }}>
          {content}
        </ButtonBase>
      ) : (
        content
      )}
      {unread && onRead ? (
        <Tooltip title={t("notifications.markread")}>
          <IconButton aria-label={t("notifications.markread")} onClick={() => onRead(n.id)} sx={{ mr: 1 }}>
            <Icon name="done_all" size="small" />
          </IconButton>
        </Tooltip>
      ) : null}
    </Stack>
  );
}
