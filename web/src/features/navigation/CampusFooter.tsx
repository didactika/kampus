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
 * The foot of every page of the campus: the links the campus sets in its footer menu (its policies, its contact…), each
 * in the language of the reader, and the version of the platform.
 */

import Box from "@mui/material/Box";
import Container from "@mui/material/Container";
import Link from "@mui/material/Link";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import NextLink from "next/link";

import client from "../../../package.json";
import type { MenuItem } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { labelOf } from "./label";

/** `<CampusFooter items={footer} />` */
export function CampusFooter({ items }: { readonly items: readonly MenuItem[] }) {
  const { t, language } = useI18n();
  const external = (target: string) => /^https?:/.test(target);
  return (
    <Box component="footer" sx={{ mt: 6, borderTop: 1, borderColor: "divider", bgcolor: "background.paper" }}>
      <Container maxWidth="lg" sx={{ py: 3 }}>
        <Stack direction={{ xs: "column", sm: "row" }} spacing={2} sx={{ alignItems: { sm: "center" }, justifyContent: "space-between" }}>
          <Stack component="nav" aria-label={t("nav.footer")} direction="row" sx={{ flexWrap: "wrap", columnGap: 3, rowGap: 1 }}>
            {items.map((item) => (
              <Link
                key={item.target + item.label}
                component={external(item.target) ? "a" : NextLink}
                href={item.target}
                {...(external(item.target) ? { target: "_blank", rel: "noopener" } : {})}
                variant="body2"
                color="text.secondary"
                underline="hover"
              >
                {labelOf(item, language)}
              </Link>
            ))}
          </Stack>
          <Typography variant="caption" color="text.secondary">
            {t("footer.version", { version: client.version })}
          </Typography>
        </Stack>
      </Container>
    </Box>
  );
}
