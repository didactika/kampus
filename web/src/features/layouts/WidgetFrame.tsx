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


"use client";

/**
 * The frame of a widget on a page being arranged: its name and icon, its settings and a way to remove it, and its
 * content below, filling the rectangle it takes, as a preview that cannot be used. The whole frame is what the
 * person drags, and it follows the pointer at its own size, so they see the whole widget they move.
 */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { forwardRef, type HTMLAttributes, type ReactNode } from "react";

import { Icon } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";

/** The props of `WidgetFrame`. */
export interface WidgetFrameProps extends HTMLAttributes<HTMLDivElement> {
  /** The name of the widget, in the language of the page. */
  readonly name: string;
  /** Its icon. */
  readonly icon: string;
  /** Whether it is the one selected. */
  readonly selected?: boolean;
  /** Whether it is the copy that follows the pointer. */
  readonly lifted?: boolean;
  /** Whether it keeps one size, which the frame says. */
  readonly fixed?: boolean;
  /** Opens its settings; none when it declares no settings. */
  readonly onSettings?: () => void;
  /** Removes it from the page. */
  readonly onRemove?: () => void;
  /** Whether it is pinned where it is, rather than rising to the widgets above it. */
  readonly pinned?: boolean;
  /** Pins it, or lets it rise again. */
  readonly onPin?: () => void;
  /** Its content, or what stands for it until the page is saved. */
  readonly children: ReactNode;
}

/** Keeps a press on a button of the frame from starting a drag of the whole frame. */
const own = {
  onPointerDown: (event: { stopPropagation: () => void }) => event.stopPropagation(),
  onKeyDown: (event: { stopPropagation: () => void }) => event.stopPropagation(),
};

/** `<WidgetFrame name="Catalogue" icon="school">{preview}</WidgetFrame>` */
export const WidgetFrame = forwardRef<HTMLDivElement, WidgetFrameProps>(function WidgetFrame(
  { name, icon, selected = false, lifted = false, fixed = false, pinned = false, onPin, onSettings, onRemove, children, style, ...rest },
  ref,
) {
  const { t } = useI18n();
  return (
    <Card
      ref={ref}
      style={style}
      {...rest}
      sx={{
        position: "relative",
        overflow: "hidden",
        blockSize: "100%",
        display: "flex",
        flexDirection: "column",
        cursor: "move",
        touchAction: "none",
        userSelect: "none",
        outline: selected || lifted ? 2 : 0,
        outlineColor: "primary.main",
        outlineOffset: 2,
        boxShadow: lifted ? "0 24px 48px -12px color-mix(in srgb, var(--k-palette-primary-main) 45%, transparent)" : selected ? 4 : 1,
        transform: lifted ? "scale(1.02) rotate(0.5deg)" : undefined,
        transition: "box-shadow 200ms cubic-bezier(0.2, 0, 0, 1), transform 200ms cubic-bezier(0.2, 0, 0, 1)",
        "&:focus-visible": { outline: 3, outlineColor: "primary.main", outlineOffset: 2 },
        "& .widget-actions": { opacity: selected ? 1 : 0, transition: "opacity 120ms" },
        "&:hover .widget-actions, &:focus-within .widget-actions": { opacity: 1 },
      }}
    >
      <Stack direction="row" spacing={1} sx={{ alignItems: "center", pl: 1.5, pr: 1, py: 0.75, borderBottom: 1, borderColor: "divider" }}>
        <Box sx={{ color: "primary.main", display: "flex" }}>
          <Icon name={icon} size="small" />
        </Box>
        <Typography variant="subtitle2" sx={{ flex: 1 }} noWrap>
          {name}
        </Typography>
        {pinned ? (
          <Box sx={{ color: "primary.main", display: "flex" }} role="img" aria-label={t("grid.pinned")}>
            <Icon name="keep" size="small" />
          </Box>
        ) : null}
        {fixed ? (
          <Tooltip title={t("grid.fixed")}>
            <Box sx={{ color: "text.secondary", display: "flex" }} role="img" aria-label={t("grid.fixed")}>
              <Icon name="lock" size="small" />
            </Box>
          </Tooltip>
        ) : null}
        {lifted ? null : (
          <Stack direction="row" className="widget-actions">
            {onPin ? (
              <Tooltip title={t(pinned ? "grid.unpin" : "grid.pin")}>
                <IconButton
                  size="small"
                  aria-label={t(pinned ? "grid.unpin" : "grid.pin")}
                  aria-pressed={pinned}
                  sx={{ color: pinned ? "primary.main" : undefined }}
                  {...own}
                  onClick={(event) => (event.stopPropagation(), onPin())}
                >
                  <Icon name={pinned ? "keep" : "keep_off"} size="small" />
                </IconButton>
              </Tooltip>
            ) : null}
            {onSettings ? (
              <Tooltip title={t("dashboard.settings")}>
                <IconButton size="small" aria-label={t("dashboard.settings")} {...own} onClick={(event) => (event.stopPropagation(), onSettings())}>
                  <Icon name="tune" size="small" />
                </IconButton>
              </Tooltip>
            ) : null}
            {onRemove ? (
              <Tooltip title={t("dashboard.remove")}>
                <IconButton size="small" aria-label={t("dashboard.remove")} sx={{ color: "error.main" }} {...own} onClick={(event) => (event.stopPropagation(), onRemove())}>
                  <Icon name="delete" size="small" />
                </IconButton>
              </Tooltip>
            ) : null}
          </Stack>
        )}
      </Stack>
      <Box sx={{ flex: 1, minBlockSize: 0, p: 2, pointerEvents: "none", opacity: 0.94, overflow: "hidden" }} inert>
        {children}
      </Box>
    </Card>
  );
});
