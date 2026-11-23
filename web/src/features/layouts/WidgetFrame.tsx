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
 * The frame of a widget on a page being arranged. The widget shows exactly as it reads, with nothing added around it:
 * an outline marks it under the pointer or when selected, and a small bar floating over its corner names it and holds
 * its actions (pin, settings, remove). The whole widget is what the person drags; its content is a preview that
 * cannot be used.
 */

import Box from "@mui/material/Box";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { forwardRef, type HTMLAttributes, type ReactNode, type Ref } from "react";

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
  /** Whether it is following the pointer. */
  readonly lifted?: boolean;
  /** Whether it keeps one size, which the frame says. */
  readonly fixed?: boolean;
  /** Whether it is pinned where it is, rather than rising to the widgets above it. */
  readonly pinned?: boolean;
  /** Pins it, or lets it rise again. */
  readonly onPin?: () => void;
  /** Opens its settings; none when it declares no settings. */
  readonly onSettings?: () => void;
  /** Removes it from the page; none when every page holds it. */
  readonly onRemove?: () => void;
  /** Where its content is, to measure the height it shows. */
  readonly contentRef?: Ref<HTMLDivElement>;
  /** The most its content shows, in pixels: the height of its rows. */
  readonly maxContent?: number;
  /** Its content, or its shape until the page is rendered again. */
  readonly children: ReactNode;
}

/** Keeps a press on a button of the frame from starting a drag of the whole frame. */
const own = {
  onPointerDown: (event: { stopPropagation: () => void }) => event.stopPropagation(),
  onKeyDown: (event: { stopPropagation: () => void }) => event.stopPropagation(),
};

/** `<WidgetFrame name="Catalogue" icon="school">{preview}</WidgetFrame>` */
export const WidgetFrame = forwardRef<HTMLDivElement, WidgetFrameProps>(function WidgetFrame(
  { name, icon, selected = false, lifted = false, fixed = false, pinned = false, onPin, onSettings, onRemove, contentRef, maxContent, children, style, ...rest },
  ref,
) {
  const { t } = useI18n();
  const action = (label: string, glyph: string, onClick: () => void, extra: object = {}) => (
    <Tooltip title={label}>
      <IconButton size="small" aria-label={label} {...own} {...extra} onClick={(event) => (event.stopPropagation(), onClick())}>
        <Icon name={glyph} size="small" />
      </IconButton>
    </Tooltip>
  );
  return (
    <Box
      ref={ref}
      style={style}
      {...rest}
      sx={{
        position: "relative",
        blockSize: "100%",
        borderRadius: 4,
        cursor: "move",
        touchAction: "none",
        userSelect: "none",
        outline: "none",
        transform: lifted ? "scale(1.01)" : undefined,
        transition: "transform 200ms cubic-bezier(0.2, 0, 0, 1)",
        // The outline is drawn on the widget itself, so it follows its own shape and the radius of its corners.
        "& .widget-content > *": {
          outline: "2px solid transparent",
          outlineOffset: -2,
          transition: "outline-color 120ms, box-shadow 200ms cubic-bezier(0.2, 0, 0, 1)",
          ...(selected || lifted ? { outlineColor: "var(--k-palette-primary-main)" } : {}),
          ...(lifted ? { boxShadow: "0 24px 48px -12px color-mix(in srgb, var(--k-palette-primary-main) 45%, transparent)" } : {}),
        },
        "&:hover .widget-content > *": selected || lifted ? {} : { outlineColor: "color-mix(in srgb, var(--k-palette-primary-main) 45%, transparent)" },
        "&:focus-visible .widget-content > *": { outlineWidth: 3, outlineColor: "var(--k-palette-primary-main)" },
        "& .widget-actions": { opacity: selected ? 1 : 0, transform: selected ? "none" : "translateY(-4px)", transition: "opacity 120ms, transform 120ms" },
        "&:hover .widget-actions, &:focus-within .widget-actions": { opacity: 1, transform: "none" },
      }}
    >
      <Box ref={contentRef} inert className="widget-content" sx={{ pointerEvents: "none", overflow: "clip", overflowClipMargin: 40, maxBlockSize: maxContent, display: "flex", flexDirection: "column", "& > *": { minBlockSize: 0 } }}>
        {children}
      </Box>
      {lifted ? null : (
        <Stack
          direction="row"
          className="widget-actions"
          sx={{
            position: "absolute",
            insetBlockStart: 8,
            insetInlineEnd: 8,
            zIndex: 2,
            alignItems: "center",
            gap: 0.25,
            pl: 1.25,
            pr: 0.5,
            py: 0.25,
            maxInlineSize: "calc(100% - 16px)",
            borderRadius: 999,
            bgcolor: "background.paper",
            border: 1,
            borderColor: "divider",
            boxShadow: 3,
            color: "text.primary",
          }}
        >
          <Box sx={{ color: "primary.main", display: "flex", flex: "none" }} aria-hidden>
            <Icon name={icon} size="small" />
          </Box>
          <Typography variant="caption" noWrap sx={{ fontWeight: 600, mx: 0.75, minInlineSize: 0 }}>
            {name}
          </Typography>
          {fixed ? (
            <Tooltip title={t("grid.fixed")}>
              <Box sx={{ color: "text.secondary", display: "flex", px: 0.5 }} role="img" aria-label={t("grid.fixed")}>
                <Icon name="lock" size="small" />
              </Box>
            </Tooltip>
          ) : null}
          {onPin ? action(t(pinned ? "grid.unpin" : "grid.pin"), pinned ? "keep" : "keep_off", onPin, { "aria-pressed": pinned, sx: { color: pinned ? "primary.main" : undefined } }) : null}
          {onSettings ? action(t("dashboard.settings"), "tune", onSettings) : null}
          {onRemove ? action(t("dashboard.remove"), "delete", onRemove, { sx: { color: "error.main" } }) : null}
        </Stack>
      )}
    </Box>
  );
});
