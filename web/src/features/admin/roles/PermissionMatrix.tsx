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
 * The matrix of permissions of a campus: a row per permission, grouped by the module that declares it, and a column
 * per role, each cell the value the role gives it. A cell opens one menu shared by the whole matrix, and a value
 * chosen is saved at once. The header of a role says whether it asks for a second factor and changes it.
 */

import Box from "@mui/material/Box";
import ButtonBase from "@mui/material/ButtonBase";
import IconButton from "@mui/material/IconButton";
import InputAdornment from "@mui/material/InputAdornment";
import ListItemIcon from "@mui/material/ListItemIcon";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { useMemo, useState } from "react";

import { Icon, TextField, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { PermissionInfo, RoleView } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { moduleName } from "../names";
import { CHOICES, type Choice, LOOK, roleName, valueName } from "./words";

/** The cell a menu is open for. */
interface Open {
  readonly anchor: HTMLElement;
  readonly role: RoleView;
  readonly permission: string;
}

/** The props of `PermissionMatrix`. */
export interface PermissionMatrixProps {
  readonly roles: readonly RoleView[];
  readonly permissions: readonly PermissionInfo[];
}

/** `<PermissionMatrix roles={roles} permissions={permissions} />` */
export function PermissionMatrix({ roles: initial, permissions }: PermissionMatrixProps) {
  const { t } = useI18n();
  const toast = useToast();
  const [roles, setRoles] = useState(initial);
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState<Open | null>(null);
  const [busy, setBusy] = useState(false);

  const groups = useMemo(() => {
    const words = query.trim().toLowerCase();
    const byModule = new Map<string, PermissionInfo[]>();
    for (const permission of permissions) {
      if (words && !`${permission.code} ${moduleName(t, permission.module)}`.toLowerCase().includes(words)) continue;
      byModule.set(permission.module, [...(byModule.get(permission.module) ?? []), permission]);
    }
    return [...byModule.entries()];
  }, [permissions, query, t]);

  const replace = (role: RoleView) => setRoles((current) => current.map((r) => (r.id === role.id ? role : r)));

  async function choose(value: Choice) {
    if (!open) return;
    const { role, permission } = open;
    setOpen(null);
    setBusy(true);
    try {
      replace(
        await browserApi<RoleView>(`/access/roles/${role.id}/permissions/${encodeURIComponent(permission)}`, {
          method: "PUT",
          body: { value: value === "none" ? null : value },
        }),
      );
      toast(t("admin.roles.saved"), "success");
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  async function factor(role: RoleView) {
    setBusy(true);
    try {
      replace(await browserApi<RoleView>(`/access/roles/${role.id}/second-factor`, { method: "PUT", body: { required: !role.second_factor_required } }));
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  const sticky = { position: "sticky", insetInlineStart: 0, zIndex: 1, bgcolor: "background.paper" } as const;
  const current: Choice = open ? ((open.role.values[open.permission] as Choice | undefined) ?? "none") : "none";
  return (
    <Stack spacing={2.5}>
      <Typography color="text.secondary">{t("admin.roles.matrix.lead")}</Typography>
      <TextField
        label={t("admin.roles.search")}
        type="search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        startAdornment={
          <InputAdornment position="start">
            <Icon name="search" size="small" />
          </InputAdornment>
        }
        sx={{ maxInlineSize: 420 }}
      />
      <Box sx={{ overflow: "auto", maxBlockSize: "70vh", border: 1, borderColor: "divider", borderRadius: 3, bgcolor: "background.paper" }}>
        <Box component="table" sx={{ borderCollapse: "separate", borderSpacing: 0, minInlineSize: "100%", "& th, & td": { borderBottom: 1, borderColor: "divider" } }}>
          <Box component="thead" sx={{ position: "sticky", top: 0, zIndex: 2, bgcolor: "background.paper" }}>
            <tr>
              <Box component="th" scope="col" sx={{ ...sticky, zIndex: 3, textAlign: "start", px: 2, py: 1.5, minInlineSize: 240 }}>
                <Typography variant="caption" color="text.secondary">
                  {t("admin.explain.permission")}
                </Typography>
              </Box>
              {roles.map((role) => (
                <Box component="th" scope="col" key={role.id} sx={{ px: 1, py: 1.5, minInlineSize: 112, verticalAlign: "bottom" }}>
                  <Stack spacing={0.5} sx={{ alignItems: "center" }}>
                    <Typography variant="caption" sx={{ fontWeight: 700, textAlign: "center", lineHeight: 1.25 }}>
                      {roleName(t, role)}
                    </Typography>
                    <Tooltip title={role.second_factor_required ? t("admin.roles.factor.off") : t("admin.roles.factor.on")}>
                      <IconButton
                        size="small"
                        aria-label={`${roleName(t, role)}: ${t("admin.roles.factor")}`}
                        aria-pressed={role.second_factor_required}
                        disabled={busy}
                        color={role.second_factor_required ? "primary" : "default"}
                        onClick={() => void factor(role)}
                      >
                        <Icon name={role.second_factor_required ? "mobile_lock_portrait" : "passkey"} size="small" />
                      </IconButton>
                    </Tooltip>
                  </Stack>
                </Box>
              ))}
            </tr>
          </Box>
          <tbody>
            {groups.map(([module, items]) => [
              <tr key={module}>
                <Box component="th" scope="colgroup" colSpan={roles.length + 1} sx={{ ...sticky, textAlign: "start", px: 2, pt: 2, pb: 1, bgcolor: "action.hover" }}>
                  <Typography variant="subtitle2">{moduleName(t, module)}</Typography>
                </Box>
              </tr>,
              ...items.map((permission) => (
                <tr key={permission.code}>
                  <Box component="th" scope="row" sx={{ ...sticky, textAlign: "start", px: 2, py: 1, fontWeight: 400 }}>
                    <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}>
                      <Typography variant="body2" sx={{ fontFamily: "monospace" }}>
                        {permission.code}
                      </Typography>
                      {permission.risks.length ? (
                        <Tooltip title={t("admin.roles.risk", { risks: permission.risks.map((r) => t(`access.risk.${r}`)).join(", ") })}>
                          <Box component="span" role="img" sx={{ display: "flex", color: "warning.main" }} tabIndex={0} aria-label={t("admin.roles.risk", { risks: permission.risks.map((r) => t(`access.risk.${r}`)).join(", ") })}>
                            <Icon name="warning" size="small" />
                          </Box>
                        </Tooltip>
                      ) : null}
                    </Stack>
                  </Box>
                  {roles.map((role) => {
                    const value = (role.values[permission.code] as Choice | undefined) ?? "none";
                    const look = LOOK[value];
                    return (
                      <Box component="td" key={role.id} sx={{ textAlign: "center", px: 0.5, py: 0.5 }}>
                        <ButtonBase
                          aria-haspopup="menu"
                          aria-label={t("admin.roles.cell", { role: roleName(t, role), permission: permission.code, value: valueName(t, value) })}
                          disabled={busy}
                          onClick={(event) => setOpen({ anchor: event.currentTarget, role, permission: permission.code })}
                          sx={{ inlineSize: 36, blockSize: 32, borderRadius: 2, color: look.color, bgcolor: look.bg, "&:hover, &:focus-visible": { outline: 2, outlineColor: "primary.main" } }}
                        >
                          <Icon name={look.icon} size="small" />
                        </ButtonBase>
                      </Box>
                    );
                  })}
                </tr>
              )),
            ])}
          </tbody>
        </Box>
      </Box>
      <Menu anchorEl={open?.anchor} open={Boolean(open)} onClose={() => setOpen(null)}>
        {CHOICES.map((choice) => (
          <MenuItem key={choice} selected={choice === current} onClick={() => void choose(choice)}>
            <ListItemIcon sx={{ color: LOOK[choice].color }}>
              <Icon name={LOOK[choice].icon} size="small" />
            </ListItemIcon>
            {valueName(t, choice)}
          </MenuItem>
        ))}
      </Menu>
    </Stack>
  );
}
