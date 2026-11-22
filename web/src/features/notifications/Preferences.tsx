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
// @package   web/ui
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------


"use client";

/**
 * Where the person hears about each type of notification: a row per type with a switch per channel. A channel the
 * campus requires stays on, and one the campus locks cannot be changed; both say why.
 */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import Stack from "@mui/material/Stack";
import Switch from "@mui/material/Switch";
import Table from "@mui/material/Table";
import TableBody from "@mui/material/TableBody";
import TableCell from "@mui/material/TableCell";
import TableHead from "@mui/material/TableHead";
import TableRow from "@mui/material/TableRow";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { useState } from "react";

import { Icon, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { Preference } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";
import { PageHeader } from "../shell/PageHeader";
import { describe } from "./describe";
import { lookOf } from "./look";

/** `<Preferences preferences={…} />` */
export function Preferences({ preferences }: { readonly preferences: readonly Preference[] }) {
  const { t } = useI18n();
  const toast = useToast();
  const [items, setItems] = useState(preferences);
  const types = [...new Set(items.map((p) => p.type))];
  const channels = [...new Set(items.map((p) => p.channel))];
  const label = (channel: string) => (t(`channel.${channel}`) === `channel.${channel}` ? channel : t(`channel.${channel}`));

  async function toggle(preference: Preference, enabled: boolean) {
    setItems((list) => list.map((p) => (p === preference ? { ...p, enabled } : p)));
    try {
      const saved = await browserApi<Preference>(`/notify/preferences/${preference.type}/${preference.channel}`, { method: "PUT", body: { enabled } });
      setItems((list) => list.map((p) => (p.type === saved.type && p.channel === saved.channel ? saved : p)));
      toast(t("prefs.saved"), "success");
    } catch {
      setItems((list) => list.map((p) => (p.type === preference.type && p.channel === preference.channel ? preference : p)));
      toast(t("error.generic"), "danger");
    }
  }

  return (
    <>
      <PageHeader title={t("prefs.title")} lead={t("prefs.lead")} back={{ href: "/notifications", label: t("notifications.title") }} />
      <Card>
        {types.length === 0 ? (
          <EmptyState icon="tune" message={t("prefs.empty")} />
        ) : (
          <Box sx={{ overflowX: "auto" }}>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell scope="col">{t("prefs.type")}</TableCell>
                  {channels.map((c) => (
                    <TableCell key={c} scope="col" align="center">
                      {label(c)}
                    </TableCell>
                  ))}
                </TableRow>
              </TableHead>
              <TableBody>
                {types.map((type) => {
                  const look = lookOf(type);
                  return (
                    <TableRow key={type} hover>
                      <TableCell>
                        <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
                          <Box sx={{ display: "grid", placeItems: "center", inlineSize: 36, blockSize: 36, flex: "none", borderRadius: 2, bgcolor: `${look.tone}.container`, color: `${look.tone}.onContainer` }}>
                            <Icon name={look.icon} size="small" />
                          </Box>
                          <Typography variant="subtitle2">{describe(t, type)}</Typography>
                        </Stack>
                      </TableCell>
                      {channels.map((channel) => {
                        const preference = items.find((p) => p.type === type && p.channel === channel);
                        if (!preference) return <TableCell key={channel} />;
                        const fixed = preference.required || preference.locked;
                        const name = `${describe(t, type)} · ${label(channel)}`;
                        const toggleSwitch = (
                          <Switch
                            checked={preference.required || preference.enabled}
                            disabled={fixed}
                            onChange={(event) => void toggle(preference, event.target.checked)}
                            slotProps={{ input: { "aria-label": name } }}
                          />
                        );
                        return (
                          <TableCell key={channel} align="center">
                            {fixed ? (
                              <Tooltip title={preference.required ? t("prefs.required") : t("prefs.locked")}>
                                <span>{toggleSwitch}</span>
                              </Tooltip>
                            ) : (
                              toggleSwitch
                            )}
                          </TableCell>
                        );
                      })}
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </Box>
        )}
      </Card>
    </>
  );
}
