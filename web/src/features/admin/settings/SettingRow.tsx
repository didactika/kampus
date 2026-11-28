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
 * One setting of a scope: its field, built from its type and bounds, where its value comes from, and the actions on
 * it: lock it for the levels below, and take the inherited value again. A setting a higher level locks reads as held.
 */

import Box from "@mui/material/Box";
import FormControlLabel from "@mui/material/FormControlLabel";
import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";

import { Icon, SelectField, StatusBadge, TextField } from "@kampus/ui";

import type { ScopeSetting } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { optionName, settingHint, settingName } from "../names";

/** What a person changed of a setting and has not saved. */
export interface Draft {
  readonly value: unknown;
  readonly locked: boolean;
}

/** Why a value of a setting cannot be saved, if it cannot. */
export function problemOf(setting: ScopeSetting, value: unknown): "range" | "length" | null {
  const kind = setting.kind;
  if (kind.type === "integer") {
    return typeof value === "number" && Number.isInteger(value) && value >= kind.min && value <= kind.max ? null : "range";
  }
  if (kind.type === "text" || kind.type === "rich_text") {
    return typeof value === "string" && [...value].length <= kind.max ? null : "length";
  }
  return null;
}

/** The props of `SettingRow`. */
export interface SettingRowProps {
  readonly setting: ScopeSetting;
  /** The scope the page shows. */
  readonly scope: string;
  /** What the person changed, if anything. */
  readonly draft: Draft | undefined;
  readonly onChange: (draft: Draft) => void;
  /** Takes the inherited value again. */
  readonly onReset: () => void;
  readonly busy: boolean;
}

/** `<SettingRow setting={s} scope={campus} draft={d} onChange={…} onReset={…} busy={false} />` */
export function SettingRow({ setting, scope, draft, onChange, onReset, busy }: SettingRowProps) {
  const { t } = useI18n();
  const name = settingName(t, setting);
  const value = draft ? draft.value : setting.value;
  const locked = draft ? draft.locked : setting.locked;
  const held = setting.locked_above;
  const change = (next: unknown) => onChange({ value: next, locked });
  const problem = problemOf(setting, value);
  const origin =
    setting.origin.from === "default"
      ? { label: t("admin.origin.default"), tone: "neutral" as const }
      : setting.origin.scope === scope
        ? { label: t("admin.origin.here"), tone: "primary" as const }
        : { label: t("admin.origin.above"), tone: "info" as const };
  const kind = setting.kind;
  const hint = settingHint(t, setting);
  let field;
  if (kind.type === "boolean") {
    field = (
      <FormControlLabel
        control={<Switch checked={value === true} disabled={held} onChange={(event) => change(event.target.checked)} />}
        label={
          <Box>
            <Typography variant="body1">{name}</Typography>
            {hint ? (
              <Typography variant="body2" color="text.secondary">
                {hint}
              </Typography>
            ) : null}
          </Box>
        }
        sx={{ alignItems: "flex-start", m: 0, gap: 1.5, "& .MuiSwitch-root": { mt: -0.5 } }}
      />
    );
  } else if (kind.type === "choice") {
    field = (
      <SelectField
        label={name}
        hint={hint}
        value={String(value ?? "")}
        disabled={held}
        onChange={change}
        options={kind.options.map((option) => ({ value: option, label: optionName(t, setting.key, option) }))}
      />
    );
  } else if (kind.type === "integer") {
    field = (
      <TextField
        label={name}
        type="number"
        value={typeof value === "number" ? String(value) : ""}
        disabled={held}
        inputProps={{ min: kind.min, max: kind.max, step: 1, inputMode: "numeric" }}
        hint={hint ?? t("admin.range", { min: kind.min, max: kind.max })}
        error={draft && problem ? t("admin.range", { min: kind.min, max: kind.max }) : undefined}
        onChange={(event) => change(event.target.value === "" ? null : Number(event.target.value))}
        sx={{ maxInlineSize: 180 }}
      />
    );
  } else {
    field = (
      <TextField
        label={name}
        value={typeof value === "string" ? value : ""}
        disabled={held}
        multiline={kind.type === "rich_text"}
        minRows={kind.type === "rich_text" ? 3 : undefined}
        hint={hint ?? t("admin.length", { max: kind.max })}
        error={draft && problem ? t("admin.length", { max: kind.max }) : undefined}
        onChange={(event) => change(event.target.value)}
        sx={{ maxInlineSize: 480 }}
      />
    );
  }
  return (
    <Box
      role="group"
      aria-label={name}
      sx={{
        display: "grid",
        gridTemplateColumns: "minmax(0, 1fr) auto",
        gap: 2,
        py: 2,
        pl: 1.5,
        borderInlineStart: 3,
        borderInlineStartColor: draft ? "primary.main" : "transparent",
        "& + &": { borderTop: 1, borderTopColor: "divider" },
      }}
    >
      <Box sx={{ minInlineSize: 0 }}>
        {field}
        <Stack direction="row" sx={{ flexWrap: "wrap", gap: 1, mt: 1, alignItems: "center" }}>
          {held ? <StatusBadge label={t("admin.locked.above")} tone="warning" /> : <StatusBadge label={origin.label} tone={origin.tone} />}
          {setting.restart_required ? <StatusBadge label={t("admin.restart")} tone="accent" /> : null}
          <Typography variant="caption" color="text.secondary" sx={{ fontFamily: "monospace" }}>
            {setting.key}
          </Typography>
        </Stack>
      </Box>
      <Stack direction="row" spacing={0.5} sx={{ alignItems: "flex-start" }}>
        {held ? (
          <Box sx={{ p: 1, color: "text.secondary", display: "flex" }} aria-hidden>
            <Icon name="lock" size="small" />
          </Box>
        ) : (
          <>
            <Tooltip title={locked ? t("admin.unlock") : t("admin.lock")}>
              <IconButton
                aria-label={t("admin.lock")}
                aria-pressed={locked}
                disabled={busy}
                color={locked ? "primary" : "default"}
                onClick={() => onChange({ value, locked: !locked })}
              >
                <Icon name={locked ? "lock" : "lock_open"} size="small" />
              </IconButton>
            </Tooltip>
            {setting.version > 0 ? (
              <Tooltip title={t("admin.reset")}>
                <IconButton aria-label={t("admin.reset")} disabled={busy} onClick={onReset}>
                  <Icon name="restart_alt" size="small" />
                </IconButton>
              </Tooltip>
            ) : null}
          </>
        )}
      </Stack>
    </Box>
  );
}
