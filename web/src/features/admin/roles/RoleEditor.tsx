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
 * A role of the campus: the value it gives each permission, grouped by the module that declares them and found by words
 * of their code, each changed with four choices and saved from the floating bar; and whether it asks for a second
 * factor.
 */

import Box from "@mui/material/Box";
import FormControlLabel from "@mui/material/FormControlLabel";
import InputAdornment from "@mui/material/InputAdornment";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { ChangesBar, Icon, SectionCard, TextField, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { PermissionInfo, RoleView } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { PageHeader } from "../../shell/PageHeader";
import { useLeaveGuard } from "../../shell/LeaveGuard";
import { moduleName } from "../names";
import { CHOICES, type Choice, LOOK, permissionName, roleName, valueName } from "./words";

/** The props of `RoleEditor`. */
export interface RoleEditorProps {
  readonly role: RoleView;
  readonly permissions: readonly PermissionInfo[];
}

/** `<RoleEditor role={role} permissions={permissions} />` */
export function RoleEditor({ role, permissions }: RoleEditorProps) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [query, setQuery] = useState("");
  const [drafts, setDrafts] = useState<ReadonlyMap<string, Choice>>(new Map());
  const [busy, setBusy] = useState(false);
  const guard = useLeaveGuard(drafts.size > 0);
  const valueOf = (code: string): Choice => drafts.get(code) ?? ((role.values[code] as Choice | undefined) ?? "none");

  const groups = useMemo(() => {
    const words = query.trim().toLowerCase();
    const byModule = new Map<string, PermissionInfo[]>();
    for (const permission of permissions) {
      if (words && !`${permission.code} ${permissionName(t, permission.code)} ${moduleName(t, permission.module)}`.toLowerCase().includes(words)) continue;
      byModule.set(permission.module, [...(byModule.get(permission.module) ?? []), permission]);
    }
    return [...byModule.entries()].sort(([a], [b]) => moduleName(t, a).localeCompare(moduleName(t, b)));
  }, [permissions, query, t]);

  const choose = (code: string, value: Choice) =>
    setDrafts((current) => {
      const next = new Map(current);
      if (value === ((role.values[code] as Choice | undefined) ?? "none")) next.delete(code);
      else next.set(code, value);
      return next;
    });

  async function save() {
    setBusy(true);
    let failed = 0;
    for (const [code, value] of drafts) {
      await browserApi(`/access/roles/${role.id}/permissions/${encodeURIComponent(code)}`, { method: "PUT", body: { value: value === "none" ? null : value } }).catch(() => {
        failed += 1;
      });
    }
    setBusy(false);
    setDrafts(new Map());
    toast(failed ? t("admin.failed", { n: failed }) : t("admin.roles.saved.all"), failed ? "danger" : "success");
    router.refresh();
  }

  async function factor(required: boolean) {
    setBusy(true);
    try {
      await browserApi(`/access/roles/${role.id}/second-factor`, { method: "PUT", body: { required } });
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Stack spacing={2.5}>
      <PageHeader
        title={roleName(t, role)}
        lead={t("admin.roles.lead", { where: role.kinds.map((k) => t(`admin.roles.kind.${k}`)).join(", ") || "—" })}
        back={{ href: "/admin/roles", label: t("admin.page.roles") }}
        actions={
          <FormControlLabel
            control={<Switch checked={role.second_factor_required} disabled={busy} onChange={(event) => void factor(event.target.checked)} />}
            label={t("admin.roles.factor.ask")}
          />
        }
      />
      <TextField
        label={t("admin.roles.search")}
        type="search"
        width="medium"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        startAdornment={
          <InputAdornment position="start">
            <Icon name="search" size="small" />
          </InputAdornment>
        }
      />
      {groups.map(([module, items]) => (
        <SectionCard key={module} title={moduleName(t, module)}>
          {items.map((permission) => {
            const value = valueOf(permission.code);
            const risks = permission.risks.map((r) => t(`access.risk.${r}`)).join(", ");
            return (
              <Box
                key={permission.code}
                sx={{
                  display: "flex",
                  alignItems: "center",
                  gap: 2,
                  flexWrap: "wrap",
                  py: 1.25,
                  "& + &": { borderTop: 1, borderColor: "divider" },
                  bgcolor: drafts.has(permission.code) ? "color-mix(in srgb, var(--k-palette-warning-main) 8%, transparent)" : "transparent",
                  mx: -1,
                  px: 1,
                  borderRadius: 1,
                }}
              >
                <Stack direction="row" spacing={1} sx={{ flex: 1, minInlineSize: 220, alignItems: "center" }}>
                  <Tooltip title={permission.code}>
                    <Typography variant="body2" sx={{ fontWeight: 500 }}>
                      {permissionName(t, permission.code)}
                    </Typography>
                  </Tooltip>
                  {risks ? (
                    <Tooltip title={t("admin.roles.risk", { risks })}>
                      <Box component="span" role="img" tabIndex={0} aria-label={t("admin.roles.risk", { risks })} sx={{ display: "flex", color: "warning.main" }}>
                        <Icon name="warning" size="small" />
                      </Box>
                    </Tooltip>
                  ) : null}
                </Stack>
                <ToggleButtonGroup
                  exclusive
                  size="small"
                  value={value}
                  aria-label={permissionName(t, permission.code)}
                  onChange={(_, next: Choice | null) => next && choose(permission.code, next)}
                  sx={{ "& .MuiToggleButton-root": { textTransform: "none", gap: 0.5, px: 1.25, fontWeight: 600 } }}
                >
                  {CHOICES.map((choice) => (
                    <ToggleButton key={choice} value={choice} sx={{ "&.Mui-selected": { color: LOOK[choice].color, bgcolor: LOOK[choice].bg } }}>
                      <Icon name={LOOK[choice].icon} size="small" />
                      {valueName(t, choice)}
                    </ToggleButton>
                  ))}
                </ToggleButtonGroup>
              </Box>
            );
          })}
        </SectionCard>
      ))}
      <ChangesBar
        open={drafts.size > 0}
        label={roleName(t, role)}
        message={t("admin.changes", { n: drafts.size })}
        saveLabel={t("admin.roles.save")}
        discardLabel={t("changes.discard")}
        busy={busy}
        onSave={() => void save()}
        onDiscard={() => setDrafts(new Map())}
      />
      {guard}
    </Stack>
  );
}
