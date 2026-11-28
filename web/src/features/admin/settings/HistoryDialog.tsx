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

/** The changes of a setting in a scope, newest first: when, who, from what value to what, and whether it was locked. */

import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useEffect, useState } from "react";

import { Button, Loading, RowList, Row, StatusBadge } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { Me, ScopeSetting } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { formatDate } from "../../../i18n/translator";
import { settingName } from "../names";
import { formatValue } from "./format";

/** A change as the server answers it. */
interface Change {
  readonly actor: string | null;
  readonly before: unknown;
  readonly after: unknown;
  readonly locked: boolean;
  readonly at: string;
}

/** The props of `HistoryDialog`. */
export interface HistoryDialogProps {
  readonly scope: string;
  /** The setting whose history is open; closed without one. */
  readonly setting: ScopeSetting | null;
  readonly onClose: () => void;
}

/** `<HistoryDialog scope={campus} setting={open} onClose={close} />` */
export function HistoryDialog({ scope, setting: open, onClose }: HistoryDialogProps) {
  const { t, language } = useI18n();
  // The setting last opened, kept while the dialog closes, so its content does not change as it fades.
  const [setting, setSetting] = useState<ScopeSetting | null>(open);
  useEffect(() => {
    if (open) setSetting(open);
  }, [open]);
  const [changes, setChanges] = useState<readonly Change[] | null>(null);
  const [names, setNames] = useState<ReadonlyMap<string, string>>(new Map());
  useEffect(() => {
    if (!open) return;
    setChanges(null);
    let live = true;
    void (async () => {
      const found = await browserApi<Change[]>(`/settings/scopes/${scope}/${encodeURIComponent(open.key)}/history`).catch(() => []);
      const actors = [...new Set(found.map((c) => c.actor).filter((a): a is string => Boolean(a)))];
      const read = await Promise.all(actors.map(async (id) => [id, await browserApi<Me>(`/identity/accounts/${id}`).then((a) => a.username ?? a.email ?? id).catch(() => id)] as const));
      if (live) {
        setNames(new Map(read));
        setChanges(found);
      }
    })();
    return () => {
      live = false;
    };
  }, [scope, open]);
  return (
    <Dialog open={Boolean(open)} onClose={onClose} fullWidth maxWidth="sm">
      <DialogTitle>{setting ? t("admin.history.of", { name: settingName(t, setting) }) : null}</DialogTitle>
      <DialogContent>
        {!changes || !setting ? (
          <Loading label={t("admin.loading")} />
        ) : changes.length === 0 ? (
          <Typography color="text.secondary">{t("admin.history.none")}</Typography>
        ) : (
          <RowList label={t("admin.history")}>
            {changes.map((change, index) => (
              <Row
                key={`${change.at}-${index}`}
                icon={change.after === null ? "restart_alt" : "edit"}
                title={
                  change.after === null
                    ? change.before === null
                      ? t("admin.history.inherited")
                      : t("admin.history.reset", { value: formatValue(t, setting, change.before) })
                    : change.before === null
                      ? t("admin.history.set", { value: formatValue(t, setting, change.after) })
                      : t("admin.history.changed", { before: formatValue(t, setting, change.before), after: formatValue(t, setting, change.after) })
                }
                meta={
                  <Stack direction="row" spacing={1} sx={{ alignItems: "center", flexWrap: "wrap" }}>
                    <span>{formatDate(language, change.at)}</span>
                    <span>{change.actor ? names.get(change.actor) ?? change.actor : t("admin.history.operator")}</span>
                    {change.locked ? <StatusBadge label={t("admin.locked.here")} tone="accent" /> : null}
                  </Stack>
                }
              />
            ))}
          </RowList>
        )}
      </DialogContent>
      <DialogActions>
        <Button variant="text" onClick={onClose}>
          {t("admin.close")}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
