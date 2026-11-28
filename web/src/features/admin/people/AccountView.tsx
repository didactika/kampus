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
 * An account of the campus: who it is and its status, with the changes of status its status allows, and in tabs its
 * profile (changed and saved from the floating bar), the roles it holds (assigned in the campus and taken back) and the
 * identifiers other systems give it.
 */

import Avatar from "@mui/material/Avatar";
import Stack from "@mui/material/Stack";
import Tab from "@mui/material/Tab";
import Tabs from "@mui/material/Tabs";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { Button, ChangesBar, ConfirmDialog, Icon, RowList, Row, SectionCard, SelectField, StatusBadge, TextField, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import { ApiError } from "../../../api/problem";
import type { GrantView, Me, Profile, RoleView } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { LANGUAGES, formatDay, formatRelative } from "../../../i18n/translator";
import { PageHeader } from "../../shell/PageHeader";
import { useLeaveGuard } from "../../shell/LeaveGuard";
import { roleName } from "../roles/words";
import { type AccountTab, type Transition, accountName, initials, statusName, statusTone, transitionsOf } from "./words";

/** An identifier a system gives the account. */
export interface Reference {
  readonly system: string;
  readonly external_id: string;
}

/** The props of `AccountView`. */
export interface AccountViewProps {
  readonly campus: string;
  readonly account: Me;
  readonly profile: Profile | null;
  readonly grants: readonly GrantView[] | null;
  readonly roles: readonly RoleView[];
  readonly references: readonly Reference[];
  readonly tab: AccountTab;
}

/** The fields of a profile an administrator changes. */
const FIELDS = ["given_name", "family_name", "display_name"] as const;

/** `<AccountView campus={campus} account={a} profile={p} grants={g} roles={r} references={x} tab="profile" />` */
export function AccountView({ campus, account, profile, grants, roles, references, tab }: AccountViewProps) {
  const { t, language } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [busy, setBusy] = useState(false);
  const [confirm, setConfirm] = useState<Transition | null>(null);
  const due = Boolean(account.deletion_due_at);

  async function change(to: Transition) {
    setBusy(true);
    try {
      await browserApi(`/identity/accounts/${account.id}/transitions`, {
        method: "POST",
        headers: { "If-Match": `"${account.version}"` },
        body: { to, reason: "" },
      });
      toast(t(`admin.people.done.${to}`), "success");
      router.refresh();
    } catch (error) {
      toast(error instanceof ApiError && error.status === 412 ? t("profile.conflict") : t("error.generic"), "danger");
    } finally {
      setBusy(false);
      setConfirm(null);
    }
  }

  const name = profile?.display_name || [profile?.given_name, profile?.family_name].filter(Boolean).join(" ") || accountName(account);
  return (
    <Stack spacing={3}>
      <PageHeader
        title={name}
        lead={account.email && account.email !== name ? account.email : null}
        leading={<Avatar sx={{ inlineSize: 64, blockSize: 64, fontSize: "1.5rem", fontWeight: 700, backgroundImage: "var(--k-palette-gradient-primary)", color: "primary.contrastText" }}>{initials(account)}</Avatar>}
        meta={
          <Stack direction="row" spacing={1} sx={{ alignItems: "center", flexWrap: "wrap" }}>
            <StatusBadge label={due ? t("admin.people.status.deletion_scheduled") : statusName(t, account.status)} tone={due ? "danger" : statusTone(account.status)} />
            <Typography variant="body2" color="text.secondary" suppressHydrationWarning>
              {account.last_sign_in_at ? t("admin.people.last.at", { when: formatRelative(language, account.last_sign_in_at) }) : t("admin.people.never")}
            </Typography>
          </Stack>
        }
        actions={
          <Stack direction="row" spacing={1}>
            {transitionsOf(account.status, due).map((to) => (
              <Button
                key={to}
                variant={to === "deletion_scheduled" ? "outlined" : "tonal"}
                tone={to === "deletion_scheduled" ? "danger" : "primary"}
                busy={busy}
                onClick={() => (to === "deletion_scheduled" || to === "suspended" ? setConfirm(to) : void change(to))}
              >
                {t(`admin.people.to.${to}`)}
              </Button>
            ))}
          </Stack>
        }
      />
      <Tabs value={tab} variant="scrollable" allowScrollButtonsMobile aria-label={name}>
        <Tab value="profile" component={Link} href={`/admin/people/${account.id}`} label={t("admin.people.tab.profile")} icon={<Icon name="person" size="small" />} iconPosition="start" />
        <Tab value="roles" component={Link} href={`/admin/people/${account.id}?tab=roles`} label={t("admin.people.tab.roles")} icon={<Icon name="shield" size="small" />} iconPosition="start" />
        <Tab value="references" component={Link} href={`/admin/people/${account.id}?tab=references`} label={t("admin.people.tab.references")} icon={<Icon name="link" size="small" />} iconPosition="start" />
      </Tabs>
      {tab === "profile" ? (
        profile ? <ProfileEditor account={account} profile={profile} /> : <Typography color="text.secondary">{t("admin.people.noprofile")}</Typography>
      ) : tab === "roles" ? (
        <RolesOf campus={campus} account={account} grants={grants} roles={roles} />
      ) : (
        <SectionCard title={t("admin.people.tab.references")} lead={t("admin.people.references.lead")}>
          {references.length ? (
            <RowList label={t("admin.people.tab.references")}>
              {references.map((reference) => (
                <Row key={reference.system} icon="link" title={reference.external_id} meta={reference.system} />
              ))}
            </RowList>
          ) : (
            <Typography color="text.secondary">{t("admin.people.references.none")}</Typography>
          )}
        </SectionCard>
      )}
      <ConfirmDialog
        open={confirm !== null}
        title={confirm ? t(`admin.people.confirm.${confirm}`, { name }) : ""}
        confirmLabel={confirm ? t(`admin.people.to.${confirm}`) : ""}
        cancelLabel={t("changes.discard")}
        tone={confirm === "deletion_scheduled" ? "danger" : "primary"}
        busy={busy}
        onConfirm={() => confirm && void change(confirm)}
        onClose={() => setConfirm(null)}
      >
        {confirm ? t(`admin.people.confirm.${confirm}.lead`) : null}
      </ConfirmDialog>
    </Stack>
  );
}

/** The profile of the account, changed from the floating bar. */
function ProfileEditor({ account, profile }: { readonly account: Me; readonly profile: Profile }) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const start = useMemo(
    () => ({ given_name: profile.given_name ?? "", family_name: profile.family_name ?? "", display_name: profile.display_name ?? "", language_code: profile.language_code ?? "" }),
    [profile],
  );
  const [values, setValues] = useState(start);
  const [busy, setBusy] = useState(false);
  const dirty = JSON.stringify(values) !== JSON.stringify(start);
  const guard = useLeaveGuard(dirty);
  const set = (field: keyof typeof start, value: string) => setValues((current) => ({ ...current, [field]: value }));
  async function save() {
    setBusy(true);
    const text = (v: string) => (v.trim() === "" ? null : v.trim());
    try {
      await browserApi<Profile>(`/users/accounts/${account.id}/profile`, {
        method: "PUT",
        headers: { "If-Match": `"${profile.version}"` },
        body: { given_name: text(values.given_name), family_name: text(values.family_name), display_name: text(values.display_name), language_code: text(values.language_code), time_zone: profile.time_zone },
      });
      toast(t("profile.saved"), "success");
      router.refresh();
    } catch (error) {
      toast(error instanceof ApiError && error.status === 412 ? t("profile.conflict") : t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }
  return (
    <SectionCard title={t("admin.people.tab.profile")}>
      <Stack spacing={2.5}>
        <Stack direction={{ xs: "column", sm: "row" }} spacing={2}>
          {FIELDS.slice(0, 2).map((field) => (
            <TextField key={field} label={t(`admin.people.field.${field}`)} width="medium" value={values[field]} onChange={(event) => set(field, event.target.value)} />
          ))}
        </Stack>
        <TextField label={t("admin.people.field.display_name")} hint={t("profile.display.hint")} width="medium" value={values.display_name} onChange={(event) => set("display_name", event.target.value)} />
        <SelectField
          label={t("profile.language")}
          width="medium"
          value={values.language_code}
          onChange={(value) => set("language_code", value)}
          options={[{ value: "", label: t("profile.language.auto") }, ...LANGUAGES.map((l) => ({ value: l, label: new Intl.DisplayNames([l], { type: "language" }).of(l) ?? l }))]}
        />
        <Stack direction={{ xs: "column", sm: "row" }} spacing={2}>
          <TextField label={t("profile.username")} width="medium" value={account.username ?? ""} disabled />
          <TextField label={t("profile.email")} width="medium" value={account.email ?? ""} disabled />
        </Stack>
      </Stack>
      <ChangesBar open={dirty} label={t("admin.people.tab.profile")} message={t("changes.unsaved")} saveLabel={t("profile.save")} discardLabel={t("changes.discard")} busy={busy} onSave={() => void save()} onDiscard={() => setValues(start)} />
      {guard}
    </SectionCard>
  );
}

/** The roles the account holds, a role assigned in the campus, and one taken back. */
function RolesOf({ campus, account, grants, roles }: { readonly campus: string; readonly account: Me; readonly grants: readonly GrantView[] | null; readonly roles: readonly RoleView[] }) {
  const { t, language } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const assignable = roles.filter((r) => r.kinds.includes("campus"));
  const [role, setRole] = useState(assignable[0]?.id ?? "");
  const [busy, setBusy] = useState(false);
  const named = (id: string) => {
    const found = roles.find((r) => r.id === id);
    return found ? roleName(t, found) : id;
  };
  async function run(work: () => Promise<unknown>, done: string) {
    setBusy(true);
    try {
      await work();
      toast(t(done), "success");
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }
  if (!grants) return <Typography color="text.secondary">{t("admin.roles.empty")}</Typography>;
  return (
    <Stack spacing={2.5}>
      <SectionCard title={t("admin.people.roles.held")}>
        {grants.length ? (
          <RowList label={t("admin.people.roles.held")}>
            {grants.map((grant) => (
              <Row
                key={grant.id}
                icon="shield"
                title={named(grant.role)}
                meta={[t(`admin.people.where.${grant.scope_kind}`), t("admin.grants.since", { date: formatDay(language, grant.valid_from) }), grant.origin_module ? t("admin.grants.module", { module: grant.origin_module }) : null].filter(Boolean).join(" · ")}
                actions={
                  grant.origin_module ? undefined : (
                    <Button variant="text" tone="danger" busy={busy} onClick={() => void run(() => browserApi(`/access/grants/${grant.id}`, { method: "DELETE" }), "admin.grants.revoked")}>
                      {t("admin.grants.revoke")}
                    </Button>
                  )
                }
              />
            ))}
          </RowList>
        ) : (
          <Typography color="text.secondary">{t("admin.people.roles.none")}</Typography>
        )}
      </SectionCard>
      <SectionCard title={t("admin.people.roles.add")} lead={t("admin.people.roles.add.lead")}>
        <Stack direction={{ xs: "column", sm: "row" }} spacing={2} sx={{ alignItems: { sm: "flex-end" } }}>
          <SelectField label={t("admin.grants.role")} width="medium" value={role} onChange={setRole} options={assignable.map((r) => ({ value: r.id, label: roleName(t, r) }))} />
          <Button
            variant="filled"
            icon="add"
            busy={busy}
            disabled={!role}
            onClick={() => void run(() => browserApi(`/access/scopes/${campus}/grants`, { method: "POST", body: { role, principal: account.id } }), "admin.grants.added")}
          >
            {t("admin.grants.add")}
          </Button>
        </Stack>
      </SectionCard>
    </Stack>
  );
}
