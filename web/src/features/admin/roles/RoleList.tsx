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

/** The roles of the campus: each with where it is assigned, how many permissions it allows and its second factor. */

import Box from "@mui/material/Box";
import ButtonBase from "@mui/material/ButtonBase";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import Link from "next/link";

import { Button, Icon, IconTile, StatusBadge } from "@kampus/ui";

import type { RoleView } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { PageHeader } from "../../shell/PageHeader";
import { roleName } from "./words";

/** `<RoleList roles={roles} />` */
export function RoleList({ roles }: { readonly roles: readonly RoleView[] }) {
  const { t } = useI18n();
  return (
    <Stack spacing={2.5}>
      <PageHeader
        title={t("admin.page.roles")}
        lead={t("admin.page.roles.lead")}
        actions={
          <Button variant="outlined" icon="grid_view" href="/admin/roles/compare">
            {t("admin.page.compare")}
          </Button>
        }
      />
      <Box sx={{ display: "grid", gap: 1.5, gridTemplateColumns: { xs: "1fr", md: "repeat(2, minmax(0, 1fr))" } }}>
        {roles.map((role) => {
          const allowed = Object.values(role.values).filter((v) => v === "allow").length;
          return (
            <ButtonBase
              key={role.id}
              component={Link}
              href={`/admin/roles/${role.id}`}
              sx={{
                display: "flex",
                justifyContent: "flex-start",
                alignItems: "center",
                gap: 1.75,
                p: 2,
                border: 1,
                borderColor: "divider",
                borderRadius: 3,
                bgcolor: "background.paper",
                textAlign: "start",
                "&:hover, &:focus-visible": { borderColor: "primary.main" },
              }}
            >
              <IconTile name="shield" />
              <Box sx={{ flex: 1, minInlineSize: 0 }}>
                <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
                  {roleName(t, role)}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {t("admin.roles.summary", { n: allowed, where: role.kinds.map((k) => t(`admin.roles.kind.${k}`)).join(", ") || "—" })}
                </Typography>
              </Box>
              {role.second_factor_required ? <StatusBadge label={t("admin.roles.factor")} tone="accent" /> : null}
              <Icon name="chevron_right" size="small" />
            </ButtonBase>
          );
        })}
      </Box>
    </Stack>
  );
}
