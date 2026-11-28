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
 * One setting: its name and what it does, its field, and what is known of its value (its default, where it comes from,
 * whether it was changed, locked or waits to be saved), with its actions in a menu: lock it for the levels below, take
 * the inherited value again, read its history. A setting the link of a search points at is brought into view.
 */

import Box from "@mui/material/Box";
import IconButton from "@mui/material/IconButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { useEffect, useId, useRef, useState } from "react";

import { Icon, StatusBadge } from "@kampus/ui";

import type { ScopeSetting } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { settingHint, settingName } from "../names";
import { formatValue, problemOf, same } from "./format";
import { SettingControl } from "./SettingControl";

/** What a person changed of a setting and has not saved. */
export interface Draft {
  readonly value: unknown;
  readonly locked: boolean;
}

/** The props of `SettingItem`. */
export interface SettingItemProps {
  readonly setting: ScopeSetting;
  /** The scope the page shows. */
  readonly scope: string;
  readonly draft: Draft | undefined;
  /** The problem the server gave when it was last saved. */
  readonly failed: string | undefined;
  readonly busy: boolean;
  readonly onChange: (draft: Draft) => void;
  readonly onReset: () => void;
  readonly onHistory: () => void;
}

/** `<SettingItem setting={s} scope={campus} … />` */
export function SettingItem({ setting, scope, draft, failed, busy, onChange, onReset, onHistory }: SettingItemProps) {
  const { t } = useI18n();
  const own = useId();
  const [menu, setMenu] = useState<HTMLElement | null>(null);
  const [pointed, setPointed] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (window.location.hash === `#${setting.key}`) {
      root.current?.scrollIntoView({ block: "center" });
      setPointed(true);
      const timer = setTimeout(() => setPointed(false), 2400);
      return () => clearTimeout(timer);
    }
    return undefined;
  }, [setting.key]);
  const value = draft ? draft.value : setting.value;
  const locked = draft ? draft.locked : setting.locked;
  const held = setting.locked_above;
  const changed = !same(value, setting.default_value);
  const problem = draft ? problemOf(setting, value) : null;
  const name = settingName(t, setting);
  const rich = setting.kind.type === "rich_text";
  const hint = settingHint(t, setting);
  const origin = setting.origin.from === "default" ? null : { inherited: setting.origin.scope !== scope };
  const ids = { name: `${own}-name`, help: `${own}-help` };
  return (
    <Box
      ref={root}
      id={setting.key}
      role="group"
      aria-labelledby={ids.name}
      sx={{
        position: "relative",
        display: "grid",
        gridTemplateColumns: { xs: "minmax(0, 1fr) auto", md: rich ? "minmax(0, 1fr) auto" : "minmax(0, 1fr) auto auto" },
        columnGap: 3,
        rowGap: 1.5,
        alignItems: "start",
        py: 2,
        px: 2,
        mx: -2,
        borderRadius: 3,
        scrollMarginTop: 96,
        transition: "background-color .6s",
        bgcolor: pointed ? "color-mix(in srgb, var(--k-palette-primary-main) 10%, transparent)" : draft ? "color-mix(in srgb, var(--k-palette-primary-main) 4%, transparent)" : "transparent",
        "&::before": {
          content: '""',
          position: "absolute",
          insetInlineStart: 0,
          top: 18,
          bottom: 18,
          inlineSize: 3,
          borderRadius: 3,
          bgcolor: draft ? "warning.main" : changed ? "primary.main" : "transparent",
        },
        "& + &": { borderTop: 1, borderTopColor: "divider", borderRadius: 0 },
      }}
    >
      <Box sx={{ minInlineSize: 0, gridColumn: "1 / 2", gridRow: 1 }}>
        <Typography id={ids.name} variant="subtitle1" component="h3" sx={{ fontWeight: 600, lineHeight: 1.35 }}>
          {name}
        </Typography>
        <Typography id={ids.help} variant="body2" color="text.secondary" sx={{ mt: 0.25 }}>
          {hint ?? setting.summary}
        </Typography>
        {draft || held || locked || origin?.inherited || changed || setting.restart_required ? (
          <Stack direction="row" sx={{ flexWrap: "wrap", gap: 0.75, mt: 1, alignItems: "center" }}>
            {draft ? <StatusBadge label={t("admin.unsaved")} tone="warning" /> : null}
            {held ? <StatusBadge label={t("admin.locked.above")} tone="warning" /> : origin?.inherited ? <StatusBadge label={t("admin.origin.above")} tone="info" /> : null}
            {locked && !held ? <StatusBadge label={t("admin.locked.here")} tone="accent" /> : null}
            {setting.restart_required ? <StatusBadge label={t("admin.restart")} tone="neutral" /> : null}
            {changed ? (
              <Typography variant="caption" color="text.secondary">
                {t("admin.default", { value: formatValue(t, setting, setting.default_value) })}
              </Typography>
            ) : null}
          </Stack>
        ) : null}
      </Box>
      <Box sx={{ gridColumn: { xs: "1 / -1", md: rich ? "1 / -1" : "2 / 3" }, gridRow: { xs: 2, md: rich ? 2 : 1 }, minInlineSize: 0, display: "flex", flexDirection: "column", alignItems: { md: "flex-end" } }}>
        <SettingControl
          setting={setting}
          value={value}
          disabled={held || busy}
          invalid={Boolean(problem)}
          labelledBy={ids.name}
          describedBy={ids.help}
          onChange={(next) => onChange({ value: next, locked })}
        />
        {problem ? (
          <Typography variant="caption" color="error" role="alert" sx={{ display: "block", mt: 0.5 }}>
            {problem === "range" && setting.kind.type === "integer"
              ? t("admin.range", { min: setting.kind.min, max: setting.kind.max })
              : setting.kind.type === "text" || setting.kind.type === "rich_text"
                ? t("admin.length", { max: setting.kind.max })
                : null}
          </Typography>
        ) : failed ? (
          <Typography variant="caption" color="error" role="alert" sx={{ display: "block", mt: 0.5 }}>
            {t("admin.failed.one")}
          </Typography>
        ) : null}
      </Box>
      <Box sx={{ gridColumn: { xs: "2 / 3", md: rich ? "2 / 3" : "3 / 4" }, gridRow: 1 }}>
        <Tooltip title={t("admin.actions")}>
          <IconButton aria-label={t("admin.actions.of", { name })} aria-haspopup="menu" onClick={(event) => setMenu(event.currentTarget)}>
            <Icon name="more_vert" size="small" />
          </IconButton>
        </Tooltip>
        <Menu anchorEl={menu} open={Boolean(menu)} onClose={() => setMenu(null)} anchorOrigin={{ vertical: "bottom", horizontal: "right" }} transformOrigin={{ vertical: "top", horizontal: "right" }}>
          <MenuItem
            disabled={held || busy}
            onClick={() => {
              setMenu(null);
              onChange({ value, locked: !locked });
            }}
          >
            <ListItemIcon>
              <Icon name={locked ? "lock_open" : "lock"} size="small" />
            </ListItemIcon>
            {locked ? t("admin.unlock") : t("admin.lock")}
          </MenuItem>
          <MenuItem
            disabled={held || busy || setting.version === 0}
            onClick={() => {
              setMenu(null);
              onReset();
            }}
          >
            <ListItemIcon>
              <Icon name="restart_alt" size="small" />
            </ListItemIcon>
            {t("admin.reset")}
          </MenuItem>
          <MenuItem
            onClick={() => {
              setMenu(null);
              onHistory();
            }}
          >
            <ListItemIcon>
              <Icon name="history" size="small" />
            </ListItemIcon>
            {t("admin.history")}
          </MenuItem>
        </Menu>
      </Box>
    </Box>
  );
}
