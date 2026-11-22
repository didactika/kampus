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

/** The recovery codes of the account, shown once: to copy or download before closing. */

import Box from "@mui/material/Box";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Typography from "@mui/material/Typography";

import { Button, useToast } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";

/** The name of the file the codes download as. */
const FILE = "kampus-recovery-codes.txt";

/** `<RecoveryCodesDialog codes={codes} onClose={() => setCodes(null)} />`; closed without codes. */
export function RecoveryCodesDialog({ codes, onClose }: { readonly codes: readonly string[] | null; readonly onClose: () => void }) {
  const { t } = useI18n();
  const toast = useToast();
  const text = (codes ?? []).join("\n");

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      toast(t("codes.copied"), "success");
    } catch {
      toast(t("error.generic"), "danger");
    }
  }

  function download() {
    const url = URL.createObjectURL(new Blob([`${text}\n`], { type: "text/plain" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = FILE;
    link.click();
    URL.revokeObjectURL(url);
  }

  return (
    <Dialog open={codes !== null} aria-labelledby="codes-title" aria-describedby="codes-lead" fullWidth maxWidth="xs">
      <DialogTitle id="codes-title">{t("codes.title")}</DialogTitle>
      <DialogContent>
        <Typography id="codes-lead" color="text.secondary" sx={{ mb: 2.5 }}>
          {t("codes.lead")}
        </Typography>
        <Box
          component="ul"
          aria-label={t("codes.title")}
          sx={{ m: 0, p: 2, listStyle: "none", display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: 1, borderRadius: 3, bgcolor: "action.hover", fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace", fontSize: "0.95rem", textAlign: "center" }}
        >
          {(codes ?? []).map((code) => (
            <li key={code}>{code}</li>
          ))}
        </Box>
      </DialogContent>
      <DialogActions sx={{ flexWrap: "wrap", gap: 1 }}>
        <Button variant="text" icon="content_copy" onClick={() => void copy()}>
          {t("codes.copy")}
        </Button>
        <Button variant="text" icon="download" onClick={download}>
          {t("codes.download")}
        </Button>
        <Button variant="filled" onClick={onClose}>
          {t("codes.done")}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
