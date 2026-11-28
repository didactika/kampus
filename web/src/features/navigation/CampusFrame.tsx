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
 * The frame of every page of a campus: a link that skips to the content, the bar of the brand with the main
 * navigation, the appearance and the account, and the content under it. On a narrow window the navigation folds into
 * a drawer.
 */

import AppBar from "@mui/material/AppBar";
import Avatar from "@mui/material/Avatar";
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
import Badge from "@mui/material/Badge";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import Stack from "@mui/material/Stack";
import Toolbar from "@mui/material/Toolbar";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { useId, useState, type ReactNode } from "react";

import { Icon } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { Me, MenuItem as MenuEntry, Profile } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { SchemeMenu } from "../../theme/SchemeMenu";
import { CampusFooter } from "./CampusFooter";
import { LanguageMenu } from "./LanguageMenu";
import { labelOf } from "./label";
import { EditModeProvider, EditModeSwitch } from "../shell/EditMode";
import { helpAddress } from "./help";
import { initialsOf, nameOf } from "../account/names";

/** A place of the main navigation. */
interface Place {
  readonly href: string;
  readonly label: string;
  readonly icon: string;
  readonly children: readonly Place[];
}

/** The icon of a place by where it leads. */
function iconOf(target: string): string {
  if (/^https?:\/\//.test(target)) return "open_in_new";
  if (target === "/") return "home";
  if (target.startsWith("/courses")) return "school";
  if (target.startsWith("/notifications")) return "notifications";
  return "arrow_forward";
}

/** The places of a menu of the campus. */
function placesOf(items: readonly MenuEntry[], language: string): Place[] {
  return items.map((item) => ({ href: item.target, label: labelOf(item, language), icon: iconOf(item.target), children: placesOf(item.children, language) }));
}

/** Whether a place holds the page shown. */
function holds(href: string, path: string): boolean {
  return href === "/" ? path === "/" : path === href || path.startsWith(`${href}/`);
}

/** The props of `CampusFrame`. */
export interface CampusFrameProps {
  /** The signed-in account. */
  readonly account: Me;
  /** The profile of the account, for its name and initials. */
  readonly profile: Profile | null;
  /** The main menu the campus set; the default places when it set none. */
  readonly menu: readonly MenuEntry[];
  /** The footer menu the campus set. */
  readonly footer?: readonly MenuEntry[];
  /** How many notifications the person has not read. */
  readonly unread?: number;
  /** Whether the person administers anything of the campus. */
  readonly administers?: boolean;
  /** The address of the handbook, for the help of each screen; no help link without it. */
  readonly help?: string | null;
  /** The page. */
  readonly children: ReactNode;
}

/** The frame of the pages of a campus. */
export function CampusFrame({ account, profile, menu, footer = [], unread = 0, administers = false, help, children }: CampusFrameProps) {
  const { t, language } = useI18n();
  const places: Place[] = menu.length
    ? placesOf(menu, language)
    : [
        { href: "/", label: t("nav.home"), icon: "home", children: [] },
        { href: "/courses", label: t("nav.courses"), icon: "school", children: [] },
        { href: "/notifications", label: t("nav.notifications"), icon: "notifications", children: [] },
      ];
  const shown: Place[] = administers ? [...places, { href: "/admin", label: t("nav.admin"), icon: "settings", children: [] }] : places;
  const [submenu, setSubmenu] = useState<{ anchor: HTMLElement; place: Place } | null>(null);
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

  /** The help and the appearance: on the bar, or in the drawer where the bar has no room for them. */
  const tools = () => (
    <>
      {helpAddress(help, language, path) ? (
        <Tooltip title={t("nav.help")}>
          <IconButton color="inherit" component="a" href={helpAddress(help, language, path) ?? undefined} target="_blank" rel="noopener" aria-label={t("nav.help")}>
            <Icon name="help" />
          </IconButton>
        </Tooltip>
      ) : null}
      <SchemeMenu />
    </>
  );

  return (
    <EditModeProvider>
      <Box sx={{ display: "flex", flexDirection: "column", minBlockSize: "100dvh" }}>
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
          backgroundImage: "var(--k-palette-gradient-bar)",
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
              {t("app.name")}
            </Typography>
          </Box>
          <Box component="nav" aria-label={t("nav.primary")} sx={{ display: { xs: "none", md: "flex" }, gap: 0.5 }}>
            {shown.map((place) => {
              const here = holds(place.href, path) || place.children.some((c) => holds(c.href, path));
              const look = {
                color: "inherit",
                borderRadius: 999,
                px: 1.75,
                bgcolor: here ? "color-mix(in srgb, currentColor 18%, transparent)" : "transparent",
                "&:hover:not(.Mui-disabled)": { bgcolor: "color-mix(in srgb, currentColor 12%, transparent)" },
              };
              return place.children.length ? (
                <Button
                  key={place.href + place.label}
                  color="inherit"
                  variant="text"
                  aria-haspopup="menu"
                  startIcon={<Icon name={place.icon} size="small" />}
                  endIcon={<Icon name="keyboard_arrow_down" size="small" />}
                  onClick={(event) => setSubmenu({ anchor: event.currentTarget, place })}
                  sx={look}
                >
                  {place.label}
                </Button>
              ) : (
                <Button
                  key={place.href + place.label}
                  component={Link}
                  href={place.href}
                  color="inherit"
                  variant="text"
                  aria-current={here ? "page" : undefined}
                  startIcon={<Icon name={place.icon} size="small" />}
                  sx={look}
                >
                  {place.label}
                </Button>
              );
            })}
            <Menu anchorEl={submenu?.anchor} open={Boolean(submenu)} onClose={() => setSubmenu(null)}>
              {submenu?.place.children.map((child) => (
                <MenuItem key={child.href + child.label} component={Link} href={child.href} onClick={() => setSubmenu(null)} selected={holds(child.href, path)}>
                  <ListItemIcon>
                    <Icon name={child.icon} size="small" />
                  </ListItemIcon>
                  {child.label}
                </MenuItem>
              ))}
            </Menu>
          </Box>
          <Box sx={{ flex: 1 }} />
          <EditModeSwitch />
          <Tooltip title={t("nav.notifications")}>
            <IconButton color="inherit" component={Link} href="/notifications" aria-label={unread ? t("nav.notifications.unread", { n: unread }) : t("nav.notifications")}>
              <Badge badgeContent={unread} max={99} color="error" overlap="circular">
                <Icon name="notifications" />
              </Badge>
            </IconButton>
          </Tooltip>
          <Box sx={{ display: { xs: "none", sm: "flex" }, alignItems: "center", gap: 1 }}>
            {tools()}
          </Box>
          <IconButton
            color="inherit"
            aria-label={t("nav.account")}
            aria-haspopup="menu"
            aria-controls={accountMenu ? menuId : undefined}
            aria-expanded={accountMenu ? true : undefined}
            onClick={(event) => setAccountMenu(event.currentTarget)}
          >
            <Avatar sx={{ inlineSize: 34, blockSize: 34, fontSize: "0.8125rem", fontWeight: 700, bgcolor: "color-mix(in srgb, currentColor 20%, transparent)", color: "inherit", border: "1px solid color-mix(in srgb, currentColor 35%, transparent)" }}>
              {initialsOf(profile, account)}
            </Avatar>
          </IconButton>
          <Menu
            id={menuId}
            anchorEl={accountMenu}
            open={Boolean(accountMenu)}
            onClose={() => setAccountMenu(null)}
            anchorOrigin={{ vertical: "bottom", horizontal: "right" }}
            transformOrigin={{ vertical: "top", horizontal: "right" }}
            slotProps={{ paper: { sx: { minInlineSize: 260, mt: 1 } } }}
          >
            <Stack direction="row" spacing={1.5} sx={{ alignItems: "center", px: 1.5, py: 1.25 }}>
              <Avatar sx={{ inlineSize: 40, blockSize: 40, fontSize: "0.875rem", fontWeight: 700, backgroundImage: "var(--k-palette-gradient-primary)", color: "primary.contrastText" }}>
                {initialsOf(profile, account)}
              </Avatar>
              <Box sx={{ minInlineSize: 0 }}>
                <Typography variant="subtitle2" noWrap>
                  {nameOf(profile, account)}
                </Typography>
                <Typography variant="caption" color="text.secondary" noWrap component="p">
                  {account.email}
                </Typography>
              </Box>
            </Stack>
            <Divider sx={{ my: 0.5 }} />
            <MenuItem component={Link} href="/account" onClick={() => setAccountMenu(null)}>
              <ListItemIcon>
                <Icon name="person" size="small" />
              </ListItemIcon>
              {t("account.title")}
            </MenuItem>
            <LanguageMenu profile={profile} onDone={() => setAccountMenu(null)} />
            <MenuItem component={Link} href="/notifications/preferences" onClick={() => setAccountMenu(null)}>
              <ListItemIcon>
                <Icon name="tune" size="small" />
              </ListItemIcon>
              {t("nav.preferences")}
            </MenuItem>
            <Divider sx={{ my: 0.5 }} />
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
            {shown.flatMap((place) => [place, ...place.children]).map((place) => (
              <ListItemButton
                key={place.href + place.label}
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
                <ListItemText primary={place.label} />
              </ListItemButton>
            ))}
          </List>
          <Divider sx={{ mx: 2, my: 1, display: { sm: "none" } }} />
          <Stack direction="row" spacing={1} sx={{ display: { xs: "flex", sm: "none" }, alignItems: "center", px: 2, py: 1 }}>
            {tools()}
          </Stack>
        </Box>
      </Drawer>
      <Container component="main" id="content" tabIndex={-1} maxWidth="lg" sx={{ py: { xs: 3, md: 5 }, outline: "none", flex: 1 }}>
        {children}
      </Container>
      <CampusFooter items={footer} administers={administers} help={helpAddress(help, language, path)} />
      </Box>
    </EditModeProvider>
  );
}
