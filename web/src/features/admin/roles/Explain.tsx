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
 * Why a person may or may not use a permission in the campus: the decision of the check, the rule that decided it,
 * and every role it considered with its value and whether a place above overrides it.
 */

import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useState } from "react";

import { Button, RowList, Row, SectionCard, SelectField, StatusBadge, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { Decision, Me, PermissionInfo, RoleView } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { AccountPicker } from "./AccountPicker";
import { type Choice, LOOK, roleName, ruleText, valueName, permissionName } from "./words";

/** The props of `Explain`. */
export interface ExplainProps {
  readonly campus: string;
  readonly roles: readonly RoleView[];
  readonly permissions: readonly PermissionInfo[];
}

/** `<Explain campus={campus} roles={roles} permissions={permissions} />` */
export function Explain({ campus, roles, permissions }: ExplainProps) {
  const { t } = useI18n();
  const toast = useToast();
  const [person, setPerson] = useState<Me | null>(null);
  const [permission, setPermission] = useState(permissions[0]?.code ?? "");
  const [decision, setDecision] = useState<Decision | null>(null);
  const [busy, setBusy] = useState(false);
  const named = (id: string) => {
    const found = roles.find((r) => r.id === id);
    return found ? roleName(t, found) : id;
  };

  async function check() {
    if (!person || !permission) return;
    setBusy(true);
    try {
      setDecision(
        await browserApi<Decision>(
          `/access/scopes/${campus}/explain?principal=${person.id}&permission=${encodeURIComponent(permission)}`,
        ),
      );
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Stack spacing={3}>
      <SectionCard title={t("admin.page.check")} icon="help">
        <Stack spacing={2}>
          <AccountPicker
            campus={campus}
            value={person}
            onChange={(account) => {
              setPerson(account);
              setDecision(null);
            }}
          />
          <Stack direction={{ xs: "column", sm: "row" }} spacing={2} sx={{ alignItems: { sm: "flex-end" } }}>
            <SelectField
              label={t("admin.explain.permission")}
              width="medium"
              value={permission}
              searchPlaceholder={t("select.search")}
              noMatch={t("select.nomatch")}
              onChange={(code) => {
                setPermission(code);
                setDecision(null);
              }}
              options={permissions.map((p) => ({ value: p.code, label: permissionName(t, p.code) }))}
            />
            <Button variant="filled" icon="search" busy={busy} disabled={!person} onClick={() => void check()} sx={{ flex: "none" }}>
              {t("admin.explain.check")}
            </Button>
          </Stack>
        </Stack>
      </SectionCard>
      {decision ? (
        <SectionCard
          title={decision.allowed ? t("admin.explain.allowed") : t("admin.explain.denied")}
          icon={decision.allowed ? "check_circle" : "block"}
          lead={ruleText(t, decision.rule)}
        >
          <Typography variant="subtitle2" sx={{ mb: 1 }}>
            {t("admin.explain.roles")}
          </Typography>
          {decision.grants.length === 0 ? (
            <Typography color="text.secondary">{t("admin.explain.noroles")}</Typography>
          ) : (
            <RowList label={t("admin.explain.roles")}>
              {decision.grants.map((grant, index) => {
                const value = (grant.value as Choice | null) ?? "none";
                return (
                  <Row
                    key={`${grant.role}-${index}`}
                    icon={LOOK[value].icon}
                    tone={value === "allow" ? "success" : value === "forbid" ? "error" : "info"}
                    title={named(grant.role)}
                    meta={grant.set_at ? t("admin.explain.override") : t("admin.explain.own")}
                    actions={<StatusBadge label={valueName(t, value)} tone={value === "allow" ? "success" : value === "forbid" ? "danger" : "neutral"} />}
                  />
                );
              })}
            </RowList>
          )}
        </SectionCard>
      ) : null}
    </Stack>
  );
}
