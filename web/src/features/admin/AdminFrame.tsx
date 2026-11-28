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
 * The frame of the administration: its head and its sections, beside the section open. On a narrow screen the sections
 * become a row of tabs above it.
 */

import Box from "@mui/material/Box";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

import { Icon } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";
import { PageHeader } from "../shell/PageHeader";
import { ADMIN_SECTIONS } from "./sections";

/** `<AdminFrame>{section}</AdminFrame>` */
export function AdminFrame({ children }: { readonly children: ReactNode }) {
  const { t } = useI18n();
  const path = usePathname();
  return (
    <>
      <PageHeader title={t("admin.title")} lead={t("admin.lead")} />
      <Box sx={{ display: "grid", gap: 3, gridTemplateColumns: { xs: "1fr", md: "220px minmax(0, 1fr)" }, alignItems: "start" }}>
        <Box component="nav" aria-label={t("admin.sections")} sx={{ position: { md: "sticky" }, top: { md: 88 } }}>
          <List disablePadding sx={{ display: { xs: "flex", md: "block" }, gap: 0.5, overflowX: "auto" }}>
            {ADMIN_SECTIONS.map((section) => {
              const current = path.startsWith(section.href);
              return (
                <ListItem key={section.href} disablePadding sx={{ inlineSize: { xs: "auto", md: "100%" } }}>
                <ListItemButton
                  component={Link}
                  href={section.href}
                  selected={current}
                  aria-current={current ? "page" : undefined}
                  sx={{ borderRadius: 999, flex: { xs: "none", md: "initial" } }}
                >
                  <ListItemIcon sx={{ minInlineSize: 36 }}>
                    <Icon name={section.icon} size="small" />
                  </ListItemIcon>
                  <ListItemText primary={t(section.label)} />
                </ListItemButton>
                </ListItem>
              );
            })}
          </List>
        </Box>
        <Box sx={{ minInlineSize: 0 }}>{children}</Box>
      </Box>
    </>
  );
}
