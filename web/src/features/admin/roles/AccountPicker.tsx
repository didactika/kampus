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
 * Chooses a person of the campus: a search by username or email that lists the first matches, and the person chosen
 * with a way to choose another.
 */

import Avatar from "@mui/material/Avatar";
import ButtonBase from "@mui/material/ButtonBase";
import InputAdornment from "@mui/material/InputAdornment";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useEffect, useState } from "react";

import { Button, Icon, TextField } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { Me } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";

/** How many people a search lists. */
const SHOWN = 6;

/** A person as a picker shows them. */
function Person({ account }: { readonly account: Me }) {
  const name = account.username ?? account.email ?? account.id;
  return (
    <Stack direction="row" spacing={1.5} sx={{ alignItems: "center", minInlineSize: 0 }}>
      <Avatar sx={{ inlineSize: 32, blockSize: 32, fontSize: "0.8125rem", bgcolor: "primary.main" }}>{name.slice(0, 1).toUpperCase()}</Avatar>
      <Stack sx={{ minInlineSize: 0 }}>
        <Typography variant="body2" noWrap sx={{ fontWeight: 600 }}>
          {name}
        </Typography>
        {account.email && account.email !== name ? (
          <Typography variant="caption" color="text.secondary" noWrap>
            {account.email}
          </Typography>
        ) : null}
      </Stack>
    </Stack>
  );
}

/** The props of `AccountPicker`. */
export interface AccountPickerProps {
  readonly campus: string;
  readonly value: Me | null;
  readonly onChange: (account: Me | null) => void;
}

/** `<AccountPicker campus={campus} value={person} onChange={setPerson} />` */
export function AccountPicker({ campus, value, onChange }: AccountPickerProps) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const [found, setFound] = useState<readonly Me[] | null>(null);
  useEffect(() => {
    const words = query.trim();
    if (!words) {
      setFound(null);
      return;
    }
    const timer = setTimeout(() => {
      browserApi<{ readonly items: readonly Me[] }>(`/identity/campuses/${campus}/accounts?limit=${SHOWN}&q=${encodeURIComponent(words)}`)
        .then((page) => setFound(page.items))
        .catch(() => setFound([]));
    }, 250);
    return () => clearTimeout(timer);
  }, [query, campus]);

  if (value) {
    return (
      <Stack direction="row" spacing={2} sx={{ alignItems: "center", p: 1.5, border: 1, borderColor: "divider", borderRadius: 2 }}>
        <Stack sx={{ flex: 1, minInlineSize: 0 }}>
          <Person account={value} />
        </Stack>
        <Button variant="text" onClick={() => onChange(null)}>
          {t("admin.account.change")}
        </Button>
      </Stack>
    );
  }
  return (
    <Stack spacing={1}>
      <TextField
        label={t("admin.account.search")}
        type="search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        startAdornment={
          <InputAdornment position="start">
            <Icon name="search" size="small" />
          </InputAdornment>
        }
      />
      {found === null ? null : found.length === 0 ? (
        <Typography variant="body2" color="text.secondary" role="status">
          {t("admin.account.none")}
        </Typography>
      ) : (
        <Stack component="ul" sx={{ listStyle: "none", m: 0, p: 0, border: 1, borderColor: "divider", borderRadius: 2, overflow: "hidden" }}>
          {found.map((account) => (
            <li key={account.id}>
              <ButtonBase
                onClick={() => onChange(account)}
                sx={{ inlineSize: "100%", justifyContent: "flex-start", px: 1.5, py: 1, "&:hover, &:focus-visible": { bgcolor: "action.hover" } }}
              >
                <Person account={account} />
              </ButtonBase>
            </li>
          ))}
        </Stack>
      )}
    </Stack>
  );
}
