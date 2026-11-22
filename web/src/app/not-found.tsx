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

/** The page of an address that leads nowhere. */

import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";

import { requestI18n } from "../i18n/server";
import { HomeButton } from "../features/shell/HomeButton";

/** Says the page does not exist and offers the way home. */
export default async function NotFound() {
  const { t } = await requestI18n();
  return (
    <Box component="main" sx={{ minBlockSize: "100dvh", display: "grid", placeItems: "center", p: 3, textAlign: "center" }}>
      <Box sx={{ display: "grid", gap: 2, justifyItems: "center", maxInlineSize: 480 }}>
        <Typography variant="h1">{t("notfound.title")}</Typography>
        <Typography color="text.secondary">{t("notfound.lead")}</Typography>
        <HomeButton label={t("nav.home")} />
      </Box>
    </Box>
  );
}
