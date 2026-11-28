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
 * The accounts of the campus: found by username or email, kept by status and kind, a page at a time. Rows are chosen
 * to change their status together; a row opens its account; a person is invited from the head of the page.
 */

import Avatar from "@mui/material/Avatar";
import Box from "@mui/material/Box";
import Checkbox from "@mui/material/Checkbox";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import InputAdornment from "@mui/material/InputAdornment";
import Stack from "@mui/material/Stack";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";

import { Button, ConfirmDialog, Icon, PillSelect, StatusBadge, TextField, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { Me } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { formatRelative } from "../../../i18n/translator";
import { PageHeader } from "../../shell/PageHeader";
import { STATUSES, type Transition, accountName, initials, statusName, statusTone } from "./words";

/** How many accounts a page holds. */
const PAGE = 25;

/** A page of accounts as the server answers it. */
interface AccountPage {
  readonly items: readonly Me[];
  readonly next: string | null;
}

/** The props of `AccountsList`. */
export interface AccountsListProps {
  readonly campus: string;
  readonly first: AccountPage;
}

/** `<AccountsList campus={campus} first={page} />` */
export function AccountsList({ campus, first }: AccountsListProps) {
  const { t, language } = useI18n();
  const toast = useToast();
  const router = useRouter();
  const [query, setQuery] = useState("");
  const [status, setStatus] = useState("all");
  const [kind, setKind] = useState("all");
  const [rows, setRows] = useState<readonly Me[]>(first.items);
  const [next, setNext] = useState<string | null>(first.next);
  const [chosen, setChosen] = useState<ReadonlySet<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [confirm, setConfirm] = useState<Transition | null>(null);
  const [inviting, setInviting] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const address = (after?: string | null) => {
    const params = new URLSearchParams({ limit: String(PAGE) });
    if (query.trim()) params.set("q", query.trim());
    if (status !== "all") params.set("status", status);
    if (kind !== "all") params.set("kind", kind);
    if (after) params.set("after", after);
    return `/identity/campuses/${campus}/accounts?${params}`;
  };
  useEffect(() => {
    if (!loaded && !query && status === "all" && kind === "all") return;
    setLoaded(true);
    const timer = setTimeout(() => {
      browserApi<AccountPage>(address())
        .then((page) => {
          setRows(page.items);
          setNext(page.next);
          setChosen(new Set());
        })
        .catch(() => toast(t("error.generic"), "danger"));
    }, 250);
    return () => clearTimeout(timer);
  }, [query, status, kind]);

  async function more() {
    const page = await browserApi<AccountPage>(address(next)).catch(() => null);
    if (page) {
      setRows((current) => [...current, ...page.items]);
      setNext(page.next);
    }
  }

  async function apply(to: Transition) {
    setBusy(true);
    try {
      const outcomes = await browserApi<{ readonly account: string; readonly problem: string | null }[]>(`/identity/campuses/${campus}/accounts/transitions`, {
        method: "POST",
        body: { accounts: [...chosen], to, reason: "" },
      });
      const failed = outcomes.filter((o) => o.problem).length;
      toast(failed ? t("admin.people.bulk.partial", { n: failed }) : t("admin.people.bulk.done", { n: outcomes.length }), failed ? "danger" : "success");
      setChosen(new Set());
      const page = await browserApi<AccountPage>(address());
      setRows(page.items);
      setNext(page.next);
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
      setConfirm(null);
    }
  }

  const all = rows.length > 0 && rows.every((r) => chosen.has(r.id));
  const toggle = (id: string) =>
    setChosen((current) => {
      const nextSet = new Set(current);
      if (nextSet.has(id)) nextSet.delete(id);
      else nextSet.add(id);
      return nextSet;
    });

  return (
    <Stack spacing={2.5}>
      <PageHeader
        title={t("admin.page.people")}
        lead={t("admin.page.people.lead")}
        actions={
          <Button variant="filled" icon="add" onClick={() => setInviting(true)}>
            {t("admin.people.invite")}
          </Button>
        }
      />
      <Stack direction={{ xs: "column", md: "row" }} spacing={1.5} sx={{ alignItems: { md: "flex-end" } }}>
        <TextField
          label={t("admin.people.search")}
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
        <PillSelect
          label={t("admin.people.status")}
          icon="filter_list"
          value={status}
          onChange={setStatus}
          options={[{ value: "all", label: t("admin.people.status.all") }, ...STATUSES.map((s) => ({ value: s, label: statusName(t, s) }))]}
        />
        <PillSelect
          label={t("admin.people.kind")}
          icon="person"
          value={kind}
          onChange={setKind}
          options={[
            { value: "all", label: t("admin.people.kind.all") },
            { value: "person", label: t("admin.people.kind.person") },
            { value: "service", label: t("admin.people.kind.service") },
          ]}
        />
      </Stack>
      {chosen.size ? (
        <Stack
          direction="row"
          spacing={1}
          role="toolbar"
          aria-label={t("admin.people.bulk")}
          sx={{ alignItems: "center", flexWrap: "wrap", gap: 1, px: 2, py: 1, borderRadius: 3, bgcolor: "color-mix(in srgb, var(--k-palette-primary-main) 8%, transparent)" }}
        >
          <Typography variant="body2" sx={{ fontWeight: 600, mr: 1 }}>
            {t("admin.people.chosen", { n: chosen.size })}
          </Typography>
          <Button variant="tonal" icon="block" busy={busy} onClick={() => void apply("suspended")}>
            {t("admin.people.to.suspended")}
          </Button>
          <Button variant="tonal" icon="check_circle" busy={busy} onClick={() => void apply("active")}>
            {t("admin.people.to.active")}
          </Button>
          <Button variant="tonal" tone="danger" icon="delete" busy={busy} onClick={() => setConfirm("deletion_scheduled")}>
            {t("admin.people.to.deletion_scheduled")}
          </Button>
          <Box sx={{ flex: 1 }} />
          <Button variant="text" onClick={() => setChosen(new Set())}>
            {t("admin.people.unchoose")}
          </Button>
        </Stack>
      ) : null}
      <Box sx={{ border: 1, borderColor: "divider", borderRadius: 3, overflowX: "auto", bgcolor: "background.paper" }}>
        <Table size="small" aria-label={t("admin.page.people")}>
          <TableHead>
            <TableRow>
              <TableCell padding="checkbox">
                <Checkbox
                  checked={all}
                  indeterminate={chosen.size > 0 && !all}
                  onChange={() => setChosen(all ? new Set() : new Set(rows.map((r) => r.id)))}
                  slotProps={{ input: { "aria-label": t("admin.people.choose.all") } }}
                />
              </TableCell>
              <TableCell>{t("admin.people.person")}</TableCell>
              <TableCell>{t("admin.people.status")}</TableCell>
              <TableCell sx={{ display: { xs: "none", md: "table-cell" } }}>{t("admin.people.kind")}</TableCell>
              <TableCell sx={{ display: { xs: "none", sm: "table-cell" } }}>{t("admin.people.last")}</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((account) => (
              <TableRow key={account.id} hover selected={chosen.has(account.id)}>
                <TableCell padding="checkbox">
                  <Checkbox checked={chosen.has(account.id)} onChange={() => toggle(account.id)} slotProps={{ input: { "aria-label": t("admin.people.choose", { name: accountName(account) }) } }} />
                </TableCell>
                <TableCell>
                  <Stack direction="row" spacing={1.5} sx={{ alignItems: "center", minInlineSize: 0 }}>
                    <Avatar sx={{ inlineSize: 32, blockSize: 32, fontSize: "0.8125rem", bgcolor: "primary.main" }}>{initials(account)}</Avatar>
                    <Box sx={{ minInlineSize: 0 }}>
                      <Typography component={Link} href={`/admin/people/${account.id}`} variant="body2" sx={{ fontWeight: 600, color: "text.primary", textDecoration: "none", "&:hover": { textDecoration: "underline" } }}>
                        {accountName(account)}
                      </Typography>
                      {account.email && account.email !== accountName(account) ? (
                        <Typography variant="caption" color="text.secondary" component="div" noWrap>
                          {account.email}
                        </Typography>
                      ) : null}
                    </Box>
                  </Stack>
                </TableCell>
                <TableCell>
                  <StatusBadge label={statusName(t, account.status)} tone={statusTone(account.status)} />
                </TableCell>
                <TableCell sx={{ display: { xs: "none", md: "table-cell" } }}>
                  <Typography variant="body2" color="text.secondary">
                    {t(`admin.people.kind.${account.kind}`)}
                  </Typography>
                </TableCell>
                <TableCell sx={{ display: { xs: "none", sm: "table-cell" } }}>
                  <Typography variant="body2" color="text.secondary" suppressHydrationWarning>
                    {account.last_sign_in_at ? formatRelative(language, account.last_sign_in_at) : t("admin.people.never")}
                  </Typography>
                </TableCell>
              </TableRow>
            ))}
            {rows.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5}>
                  <Typography color="text.secondary" sx={{ py: 3, textAlign: "center" }} role="status">
                    {t("admin.people.none")}
                  </Typography>
                </TableCell>
              </TableRow>
            ) : null}
          </TableBody>
        </Table>
      </Box>
      {next ? (
        <Box sx={{ textAlign: "center" }}>
          <Button variant="outlined" icon="keyboard_arrow_down" onClick={() => void more()}>
            {t("admin.people.more")}
          </Button>
        </Box>
      ) : null}
      <ConfirmDialog
        open={confirm !== null}
        title={t("admin.people.delete.title", { n: chosen.size })}
        confirmLabel={t("admin.people.to.deletion_scheduled")}
        cancelLabel={t("changes.discard")}
        busy={busy}
        onConfirm={() => void apply("deletion_scheduled")}
        onClose={() => setConfirm(null)}
      >
        {t("admin.people.delete.lead")}
      </ConfirmDialog>
      <InviteDialog campus={campus} open={inviting} onClose={() => setInviting(false)} onDone={(id) => router.push(`/admin/people/${id}`)} />
    </Stack>
  );
}

/** Invites a person by email, with a username when it should not be the email. */
function InviteDialog({ campus, open, onClose, onDone }: { readonly campus: string; readonly open: boolean; readonly onClose: () => void; readonly onDone: (id: string) => void }) {
  const { t } = useI18n();
  const toast = useToast();
  const [email, setEmail] = useState("");
  const [username, setUsername] = useState("");
  const [busy, setBusy] = useState(false);
  async function invite() {
    setBusy(true);
    try {
      const made = await browserApi<{ readonly id: string }>(`/identity/campuses/${campus}/accounts`, {
        method: "POST",
        body: { email: email.trim(), username: username.trim() || null },
      });
      toast(t("admin.people.invited"), "success");
      onDone(made.id);
    } catch {
      toast(t("admin.people.invite.failed"), "danger");
    } finally {
      setBusy(false);
    }
  }
  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="xs">
      <DialogTitle>{t("admin.people.invite")}</DialogTitle>
      <DialogContent>
        <Stack spacing={2} sx={{ pt: 1 }}>
          <Typography variant="body2" color="text.secondary">
            {t("admin.people.invite.lead")}
          </Typography>
          <TextField label={t("admin.people.email")} type="email" required autoFocus value={email} onChange={(event) => setEmail(event.target.value)} />
          <TextField label={t("admin.people.username")} hint={t("admin.people.username.hint")} value={username} onChange={(event) => setUsername(event.target.value)} />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="text" onClick={onClose}>
          {t("changes.discard")}
        </Button>
        <Button variant="filled" icon="mail" busy={busy} disabled={!email.includes("@")} onClick={() => void invite()}>
          {t("admin.people.invite.send")}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
