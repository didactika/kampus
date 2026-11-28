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
 * The frame of the administration: a side panel with a search over every page and setting and the map of its
 * categories, and the page open beside it under its trail. On a narrow screen the panel opens from a button above the
 * page.
 */

import Box from "@mui/material/Box";
import Breadcrumbs from "@mui/material/Breadcrumbs";
import Drawer from "@mui/material/Drawer";
import InputAdornment from "@mui/material/InputAdornment";
import InputBase from "@mui/material/InputBase";
import MuiLink from "@mui/material/Link";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useMemo, useState, type ReactNode } from "react";

import { Button, Icon } from "@kampus/ui";

import type { ScopeSetting } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { settingHint, settingName } from "./names";
import { adminMap, pageOf, settingsHref, type AdminCategory } from "./sections";

/** How many settings a search lists. */
const SHOWN = 12;

/** The props of `AdminFrame`. */
export interface AdminFrameProps {
  /** The settings of the campus the person may change, for the map and the search. */
  readonly settings: readonly ScopeSetting[];
  readonly children: ReactNode;
}

/** The map of categories and pages. */
function MapList({ map, path, onGo }: { readonly map: readonly AdminCategory[]; readonly path: string; readonly onGo: () => void }) {
  const { t } = useI18n();
  const current = pageOf(map, path)?.page.href;
  const item = (href: string, label: string, icon: string) => (
    <ListItem key={href} disablePadding>
      <ListItemButton
        component={Link}
        href={href}
        selected={href === current || (href === "/admin" && path === "/admin")}
        aria-current={href === current ? "page" : undefined}
        onClick={onGo}
        sx={{ borderRadius: 2, py: 0.75 }}
      >
        <ListItemIcon sx={{ minInlineSize: 34 }}>
          <Icon name={icon} size="small" />
        </ListItemIcon>
        <ListItemText primary={label} slotProps={{ primary: { variant: "body2", sx: { fontWeight: 500 } } }} />
      </ListItemButton>
    </ListItem>
  );
  return (
    <>
      <List disablePadding>{item("/admin", t("admin.page.overview"), "dashboard")}</List>
      {map.map((category) =>
        category.pages.length ? (
          <Box key={category.id} sx={{ mt: 2 }}>
            <Typography variant="overline" color="text.secondary" sx={{ px: 1.5, display: "block", lineHeight: 2 }} id={`admin-${category.id}`}>
              {t(category.label)}
            </Typography>
            <List disablePadding aria-labelledby={`admin-${category.id}`}>
              {category.pages.map((page) => item(page.href, t(page.label), page.icon))}
            </List>
          </Box>
        ) : null,
      )}
    </>
  );
}

/** What a search finds: the pages whose name and the settings whose name, help or key hold its words. */
function Results({ map, settings, query, onGo }: { readonly map: readonly AdminCategory[]; readonly settings: readonly ScopeSetting[]; readonly query: string; readonly onGo: () => void }) {
  const { t } = useI18n();
  const words = query.trim().toLowerCase();
  const pages = map.flatMap((c) => c.pages).filter((p) => t(p.label).toLowerCase().includes(words));
  const found = settings
    .filter((s) => [settingName(t, s), settingHint(t, s) ?? "", s.key].some((text) => text.toLowerCase().includes(words)))
    .slice(0, SHOWN);
  if (!pages.length && !found.length) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ px: 1.5, py: 2 }} role="status">
        {t("admin.search.none", { q: query.trim() })}
      </Typography>
    );
  }
  return (
    <List disablePadding aria-label={t("admin.search.results")}>
      {pages.map((page) => (
        <ListItem key={page.href} disablePadding>
          <ListItemButton component={Link} href={page.href} onClick={onGo} sx={{ borderRadius: 2 }}>
            <ListItemIcon sx={{ minInlineSize: 34 }}>
              <Icon name={page.icon} size="small" />
            </ListItemIcon>
            <ListItemText primary={t(page.label)} secondary={t("admin.search.page")} slotProps={{ primary: { variant: "body2", sx: { fontWeight: 600 } } }} />
          </ListItemButton>
        </ListItem>
      ))}
      {found.map((setting) => (
        <ListItem key={setting.key} disablePadding>
          <ListItemButton component={Link} href={`${settingsHref(setting.module)}#${setting.key}`} onClick={onGo} sx={{ borderRadius: 2 }}>
            <ListItemIcon sx={{ minInlineSize: 34 }}>
              <Icon name="tune" size="small" />
            </ListItemIcon>
            <ListItemText
              primary={settingName(t, setting)}
              secondary={t(`admin.module.${setting.module}`)}
              slotProps={{ primary: { variant: "body2", sx: { fontWeight: 600 } } }}
            />
          </ListItemButton>
        </ListItem>
      ))}
    </List>
  );
}

/** `<AdminFrame settings={settings}>{page}</AdminFrame>` */
export function AdminFrame({ settings, children }: AdminFrameProps) {
  const { t } = useI18n();
  const path = usePathname();
  const [query, setQuery] = useState("");
  const [drawer, setDrawer] = useState(false);
  const map = useMemo(() => adminMap([...new Set(settings.map((s) => s.module))].sort((a, b) => t(`admin.module.${a}`).localeCompare(t(`admin.module.${b}`)))), [settings, t]);
  const here = pageOf(map, path);
  const go = () => {
    setDrawer(false);
    setQuery("");
  };
  const panel = (
    <Box component="nav" aria-label={t("admin.sections")} sx={{ p: 1.5 }}>
      <Box sx={{ display: "flex", alignItems: "center", gap: 1, px: 1.5, py: 1, mb: 1.5, border: 1, borderColor: "divider", borderRadius: 999, bgcolor: "background.paper", "&:focus-within": { borderColor: "primary.main", boxShadow: "0 0 0 3px color-mix(in srgb, var(--k-palette-primary-main) 20%, transparent)" } }}>
        <InputBase
          fullWidth
          type="search"
          placeholder={t("admin.search.all")}
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          inputProps={{ "aria-label": t("admin.search.all") }}
          startAdornment={
            <InputAdornment position="start">
              <Icon name="search" size="small" />
            </InputAdornment>
          }
          sx={{ typography: "body2" }}
        />
      </Box>
      {query.trim() ? <Results map={map} settings={settings} query={query} onGo={go} /> : <MapList map={map} path={path} onGo={go} />}
    </Box>
  );
  return (
    <Box sx={{ display: "grid", gap: { xs: 2, md: 4 }, gridTemplateColumns: { xs: "minmax(0, 1fr)", md: "264px minmax(0, 1fr)" }, alignItems: "start" }}>
      <Box sx={{ display: { xs: "none", md: "block" }, position: "sticky", top: 88, maxBlockSize: "calc(100dvh - 104px)", overflowY: "auto", border: 1, borderColor: "divider", borderRadius: 4, bgcolor: "background.paper" }}>
        {panel}
      </Box>
      <Box sx={{ minInlineSize: 0 }}>
        <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 1 }}>
          <Box sx={{ display: { md: "none" } }}>
            <Button variant="outlined" icon="menu" onClick={() => setDrawer(true)}>
              {t("admin.sections")}
            </Button>
          </Box>
          <Breadcrumbs aria-label={t("admin.trail")} sx={{ typography: "body2" }}>
            <MuiLink component={Link} href="/admin" variant="body2" color="text.secondary" underline="hover">
              {t("admin.title")}
            </MuiLink>
            {here ? (
              <Typography variant="body2" color="text.secondary">
                {t(here.category.label)}
              </Typography>
            ) : null}
          </Breadcrumbs>
        </Box>
        {children}
      </Box>
      <Drawer open={drawer} onClose={() => setDrawer(false)} slotProps={{ paper: { sx: { inlineSize: 300 } } }}>
        {panel}
      </Drawer>
    </Box>
  );
}
