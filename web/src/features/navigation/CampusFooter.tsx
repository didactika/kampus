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
 * The foot of every page of the campus: the brand with what the campus is, the places of the campus, what belongs to
 * the person, the links the campus sets in its footer menu (its policies, its contact…) and the help; under them the
 * year, the version of the platform and the language in use.
 */

import Box from "@mui/material/Box";
import Container from "@mui/material/Container";
import Link from "@mui/material/Link";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import NextLink from "next/link";
import type { ReactNode } from "react";

import { Icon } from "@kampus/ui";

import client from "../../../package.json";
import type { MenuItem } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { labelOf } from "./label";

/** A link of the foot. */
interface FootLink {
  readonly href: string;
  readonly label: string;
}

/** The props of `CampusFooter`. */
export interface CampusFooterProps {
  /** The footer menu the campus set. */
  readonly items: readonly MenuItem[];
  /** Whether the person administers something, so the administration is offered. */
  readonly administers?: boolean;
  /** The address of the help of the page, when the installation has a handbook. */
  readonly help?: string | null;
}

/** A column of links under its heading. */
function Column({ title, links }: { readonly title: string; readonly links: readonly FootLink[] }) {
  if (!links.length) return null;
  const external = (href: string) => /^https?:/.test(href);
  return (
    <Box component="nav" aria-label={title}>
      <Typography variant="overline" color="text.secondary" component="h2" sx={{ display: "block", mb: 1, lineHeight: 2 }}>
        {title}
      </Typography>
      <Stack component="ul" spacing={1} sx={{ listStyle: "none", m: 0, p: 0 }}>
        {links.map((link) => (
          <li key={link.href + link.label}>
            <Link
              component={external(link.href) ? "a" : NextLink}
              href={link.href}
              {...(external(link.href) ? { target: "_blank", rel: "noopener" } : {})}
              variant="body2"
              color="text.primary"
              underline="hover"
              sx={{ display: "inline-flex", alignItems: "center", gap: 0.5 }}
            >
              {link.label}
              {external(link.href) ? <Icon name="open_in_new" size="small" /> : null}
            </Link>
          </li>
        ))}
      </Stack>
    </Box>
  );
}

/** `<CampusFooter items={footer} administers help={address} />` */
export function CampusFooter({ items, administers = false, help = null }: CampusFooterProps) {
  const { t, language } = useI18n();
  const languageName = (() => {
    const name = new Intl.DisplayNames([language], { type: "language" }).of(language) ?? language;
    return name.charAt(0).toLocaleUpperCase(language) + name.slice(1);
  })();
  const bottom: ReactNode[] = [
    t("footer.copyright", { year: new Date().getFullYear() }),
    t("footer.version", { version: client.version }),
    <Box component="span" key="language" sx={{ display: "inline-flex", alignItems: "center", gap: 0.5 }}>
      <Icon name="translate" size="small" />
      {languageName}
    </Box>,
  ];
  return (
    <Box component="footer" sx={{ mt: 8, borderTop: 1, borderColor: "divider", bgcolor: "background.paper" }}>
      <Container maxWidth="lg" sx={{ pt: { xs: 4, md: 5 }, pb: 3 }}>
        <Box sx={{ display: "grid", gap: { xs: 3, md: 4 }, gridTemplateColumns: { xs: "1fr 1fr", md: "minmax(0, 1.4fr) repeat(4, minmax(0, 1fr))" } }}>
          <Box sx={{ gridColumn: { xs: "1 / -1", md: "auto" } }}>
            <Stack direction="row" spacing={1.25} sx={{ alignItems: "center", mb: 1.25 }}>
              <Box component="img" src="/kampus-icon.svg" alt="" sx={{ inlineSize: 32, blockSize: 32 }} />
              <Typography variant="h4" component="p" sx={{ fontWeight: 700 }}>
                {t("app.name")}
              </Typography>
            </Stack>
            <Typography variant="body2" color="text.secondary" sx={{ maxInlineSize: "34ch" }}>
              {t("footer.about")}
            </Typography>
          </Box>
          <Column
            title={t("footer.campus")}
            links={[
              { href: "/", label: t("nav.home") },
              { href: "/courses", label: t("nav.courses") },
              { href: "/notifications", label: t("nav.notifications") },
            ]}
          />
          <Column
            title={t("footer.you")}
            links={[
              { href: "/account", label: t("account.title") },
              { href: "/notifications/preferences", label: t("nav.preferences") },
              ...(administers ? [{ href: "/admin", label: t("nav.admin") }] : []),
            ]}
          />
          <Column title={t("footer.links")} links={items.map((item) => ({ href: item.target, label: labelOf(item, language) }))} />
          <Column title={t("footer.help")} links={help ? [{ href: help, label: t("footer.handbook") }] : []} />
        </Box>
        <Stack
          direction={{ xs: "column", sm: "row" }}
          spacing={{ xs: 0.75, sm: 2 }}
          sx={{ mt: { xs: 3, md: 4 }, pt: 2.5, borderTop: 1, borderColor: "divider", color: "text.secondary", typography: "caption", alignItems: { sm: "center" } }}
        >
          {bottom.map((part, index) => (
            <Box component="span" key={index}>
              {part}
            </Box>
          ))}
        </Stack>
      </Container>
    </Box>
  );
}
