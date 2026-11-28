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
 * The first page of the administration: a card for each category with its pages, and for the settings how many each
 * module has and how many the campus changed.
 */

import Box from "@mui/material/Box";
import ButtonBase from "@mui/material/ButtonBase";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import Link from "next/link";
import { useMemo } from "react";

import { Icon, IconTile, SectionCard } from "@kampus/ui";

import type { ScopeSetting } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { PageHeader } from "../shell/PageHeader";
import { same } from "./settings/format";
import { adminMap } from "./sections";

/** `<AdminOverview settings={settings} />` */
export function AdminOverview({ settings }: { readonly settings: readonly ScopeSetting[] }) {
  const { t } = useI18n();
  const modules = useMemo(() => [...new Set(settings.map((s) => s.module))].sort((a, b) => t(`admin.module.${a}`).localeCompare(t(`admin.module.${b}`))), [settings, t]);
  const map = adminMap(modules);
  const counts = (module: string) => {
    const own = settings.filter((s) => s.module === module);
    return { all: own.length, changed: own.filter((s) => !same(s.value, s.default_value)).length };
  };
  return (
    <Stack spacing={3}>
      <PageHeader title={t("admin.title")} lead={t("admin.lead")} />
      {map.map((category) =>
        category.pages.length ? (
          <SectionCard key={category.id} title={t(category.label)} lead={t(`${category.label}.lead`)}>
            <Box sx={{ display: "grid", gap: 1.5, gridTemplateColumns: { xs: "1fr", sm: "repeat(2, minmax(0, 1fr))", lg: "repeat(3, minmax(0, 1fr))" } }}>
              {category.pages.map((page) => {
                const module = page.href.startsWith("/admin/settings/") ? decodeURIComponent(page.href.split("/").pop()!) : null;
                const count = module ? counts(module) : null;
                return (
                  <ButtonBase
                    key={page.href}
                    component={Link}
                    href={page.href}
                    sx={{
                      display: "flex",
                      justifyContent: "flex-start",
                      alignItems: "center",
                      gap: 1.5,
                      p: 1.75,
                      border: 1,
                      borderColor: "divider",
                      borderRadius: 3,
                      textAlign: "start",
                      transition: "border-color .15s, background-color .15s",
                      "&:hover, &:focus-visible": { borderColor: "primary.main", bgcolor: "color-mix(in srgb, var(--k-palette-primary-main) 5%, transparent)" },
                    }}
                  >
                    <IconTile name={page.icon} />
                    <Box sx={{ flex: 1, minInlineSize: 0 }}>
                      <Typography variant="subtitle2" sx={{ fontWeight: 700 }}>
                        {t(page.label)}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {count ? t("admin.count.short", { n: count.all, changed: count.changed }) : t(`${page.label}.short`)}
                      </Typography>
                    </Box>
                    <Icon name="chevron_right" size="small" />
                  </ButtonBase>
                );
              })}
            </Box>
          </SectionCard>
        ) : null,
      )}
    </Stack>
  );
}
