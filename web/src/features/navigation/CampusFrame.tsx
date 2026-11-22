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
 * The frame of every page of a campus: a link that skips to the content, the bar of the brand with the main
 * navigation, the appearance and the account, and the content under it. On a narrow window the navigation folds into
 * a drawer.
 */

import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Container from "@mui/material/Container";
import Divider from "@mui/material/Divider";
import Drawer from "@mui/material/Drawer";
import IconButton from "@mui/material/IconButton";
import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useId, useState, type ReactNode } from "react";

import { Icon } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { Me } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { SchemeMenu } from "../../theme/SchemeMenu";

/** The places of the main navigation. */
const PLACES = [
  { href: "/", label: "nav.courses", icon: "school" },
  { href: "/notifications", label: "nav.notifications", icon: "notifications" },
] as const;

/** Whether a place holds the page shown. */
function holds(href: string, path: string): boolean {
  return href === "/" ? path === "/" || path.startsWith("/courses") : path.startsWith(href);
}

/** The props of `CampusFrame`. */
export interface CampusFrameProps {
  /** The signed-in account. */
  readonly account: Me;
  /** The page. */
  readonly children: ReactNode;
}

/** The frame of the pages of a campus. */
export function CampusFrame({ account, children }: CampusFrameProps) {
  const { t } = useI18n();
  const path = usePathname();
  const router = useRouter();
  const [drawer, setDrawer] = useState(false);
  const [accountMenu, setAccountMenu] = useState<HTMLElement | null>(null);
  const menuId = useId();

  async function signOut() {
    setAccountMenu(null);
    try {
      await browserApi("/identity/auth/sign-out", { method: "POST" });
    } finally {
      router.replace("/sign-in");
      router.refresh();
    }
  }

  return (
    <>
      <Box
        component="a"
        href="#content"
        sx={{
          position: "absolute",
          insetInlineStart: 8,
          top: -48,
          zIndex: "tooltip",
          px: 2,
          py: 1,
          borderRadius: 1,
          bgcolor: "background.paper",
          color: "text.primary",
          "&:focus": { top: 8 },
        }}
      >
        {t("skip")}
      </Box>
      <AppBar
        position="sticky"
        sx={{
          background: "linear-gradient(135deg, var(--k-palette-brand-main), color-mix(in srgb, var(--k-palette-brand-main) 55%, var(--k-palette-surface-scrim)))",
          color: "brand.contrastText",
        }}
      >
        <Toolbar sx={{ gap: 1 }}>
          <IconButton color="inherit" aria-label={t("nav.primary")} onClick={() => setDrawer(true)} sx={{ display: { md: "none" } }}>
            <Icon name="menu" />
          </IconButton>
          <Box component={Link} href="/" sx={{ display: "flex", alignItems: "center", gap: 1.5, color: "inherit", textDecoration: "none", mr: 3 }}>
            <Box component="img" src="/kampus-icon.svg" alt="" sx={{ inlineSize: 32, blockSize: 32 }} />
            <Typography component="span" variant="h4" sx={{ fontWeight: 600 }}>
              Kampus
            </Typography>
          </Box>
          <Box component="nav" aria-label={t("nav.primary")} sx={{ display: { xs: "none", md: "flex" }, gap: 0.5 }}>
            {PLACES.map((place) => {
              const here = holds(place.href, path);
              return (
                <Button
                  key={place.href}
                  component={Link}
                  href={place.href}
                  color="inherit"
                  variant="text"
                  aria-current={here ? "page" : undefined}
                  startIcon={<Icon name={place.icon} size="small" />}
                  sx={{ bgcolor: here ? "color-mix(in srgb, currentColor 16%, transparent)" : "transparent" }}
                >
                  {t(place.label)}
                </Button>
              );
            })}
          </Box>
          <Box sx={{ flex: 1 }} />
          <SchemeMenu />
          <IconButton
            color="inherit"
            aria-label={t("nav.account")}
            aria-haspopup="menu"
            aria-controls={accountMenu ? menuId : undefined}
            aria-expanded={accountMenu ? true : undefined}
            onClick={(event) => setAccountMenu(event.currentTarget)}
          >
            <Icon name="person" />
          </IconButton>
          <Menu id={menuId} anchorEl={accountMenu} open={Boolean(accountMenu)} onClose={() => setAccountMenu(null)}>
            <Box sx={{ px: 2, py: 1 }}>
              <Typography variant="subtitle2">{account.username}</Typography>
              <Typography variant="caption" color="text.secondary">
                {account.email}
              </Typography>
            </Box>
            <Divider />
            <MenuItem onClick={() => void signOut()}>
              <ListItemIcon>
                <Icon name="logout" size="small" />
              </ListItemIcon>
              {t("nav.signout")}
            </MenuItem>
          </Menu>
        </Toolbar>
      </AppBar>
      <Drawer open={drawer} onClose={() => setDrawer(false)}>
        <Box component="nav" aria-label={t("nav.primary")} sx={{ inlineSize: 280, pt: 2 }}>
          <List>
            {PLACES.map((place) => (
              <ListItemButton
                key={place.href}
                component={Link}
                href={place.href}
                selected={holds(place.href, path)}
                aria-current={holds(place.href, path) ? "page" : undefined}
                onClick={() => setDrawer(false)}
                sx={{ mx: 1, borderRadius: 999 }}
              >
                <ListItemIcon>
                  <Icon name={place.icon} />
                </ListItemIcon>
                <ListItemText primary={t(place.label)} />
              </ListItemButton>
            ))}
          </List>
        </Box>
      </Drawer>
      <Container component="main" id="content" tabIndex={-1} maxWidth="lg" sx={{ py: { xs: 3, md: 5 }, outline: "none" }}>
        {children}
      </Container>
    </>
  );
}
