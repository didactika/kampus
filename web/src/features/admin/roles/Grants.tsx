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
 * The roles held in the campus itself: who holds which, since when and until when, and whether a module gave it; a role
 * is assigned to a person found by name or email, and taken back from the list.
 */

import IconButton from "@mui/material/IconButton";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useState } from "react";

import { Button, Icon, RowList, Row, SectionCard, SelectField, StatusBadge, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { GrantView, Me, RoleView } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { formatDay } from "../../../i18n/translator";
import { AccountPicker } from "./AccountPicker";
import { roleName } from "./words";

/** A grant with the account that holds it, when it could be read. */
export interface Held {
  readonly grant: GrantView;
  readonly account: Me | null;
}

/** The props of `Grants`. */
export interface GrantsProps {
  readonly campus: string;
  readonly roles: readonly RoleView[];
  readonly held: readonly Held[];
}

/** `<Grants campus={campus} roles={roles} held={held} />` */
export function Grants({ campus, roles, held }: GrantsProps) {
  const { t, language } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const assignable = roles.filter((r) => r.kinds.includes("campus"));
  const [person, setPerson] = useState<Me | null>(null);
  const [role, setRole] = useState(assignable[0]?.id ?? "");
  const [busy, setBusy] = useState(false);
  const named = (id: string) => {
    const found = roles.find((r) => r.id === id);
    return found ? roleName(t, found) : id;
  };

  async function assign() {
    if (!person || !role) return;
    setBusy(true);
    try {
      await browserApi(`/access/scopes/${campus}/grants`, { method: "POST", body: { role, principal: person.id } });
      setPerson(null);
      toast(t("admin.grants.added"), "success");
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  async function revoke(grant: GrantView) {
    setBusy(true);
    try {
      await browserApi(`/access/grants/${grant.id}`, { method: "DELETE" });
      toast(t("admin.grants.revoked"), "success");
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Stack spacing={3}>
      <SectionCard title={t("admin.grants.add")} icon="person">
        <Stack spacing={2}>
          <AccountPicker campus={campus} value={person} onChange={setPerson} />
          <Stack direction={{ xs: "column", sm: "row" }} spacing={2} sx={{ alignItems: { sm: "flex-end" } }}>
            <SelectField
              label={t("admin.grants.role")}
              width="medium"
              value={role}
              onChange={setRole}
              options={assignable.map((r) => ({ value: r.id, label: roleName(t, r) }))}
            />
            <Button variant="filled" icon="add" busy={busy} disabled={!person || !role} onClick={() => void assign()} sx={{ flex: "none" }}>
              {t("admin.grants.add")}
            </Button>
          </Stack>
        </Stack>
      </SectionCard>
      {held.length === 0 ? (
        <Typography color="text.secondary">{t("admin.grants.none")}</Typography>
      ) : (
        <RowList label={t("admin.page.grants")}>
          {held.map(({ grant, account }) => (
            <Row
              key={grant.id}
              icon="person"
              title={account?.username ?? account?.email ?? grant.principal}
              meta={
                <Stack direction="row" spacing={1} sx={{ flexWrap: "wrap", alignItems: "center", rowGap: 0.5 }}>
                  <StatusBadge label={named(grant.role)} tone="primary" />
                  <span>{t("admin.grants.since", { date: formatDay(language, grant.valid_from) })}</span>
                  {grant.valid_until ? <span>{t("admin.grants.until", { date: formatDay(language, grant.valid_until) })}</span> : null}
                  {grant.origin_module ? <span>{t("admin.grants.module", { module: grant.origin_module })}</span> : null}
                </Stack>
              }
              actions={
                grant.origin_module ? undefined : (
                <Tooltip title={t("admin.grants.revoke")}>
                  <IconButton aria-label={t("admin.grants.revoke")} disabled={busy} onClick={() => void revoke(grant)}>
                    <Icon name="delete" size="small" />
                  </IconButton>
                </Tooltip>
                )
              }
            />
          ))}
        </RowList>
      )}
    </Stack>
  );
}
