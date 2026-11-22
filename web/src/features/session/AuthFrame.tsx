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


/** The frame of every page a person reaches before signing in: the brand on one side, the form on the other. */

import Box from "@mui/material/Box";
import type { ReactNode } from "react";

import { SchemeMenu } from "../../theme/SchemeMenu";
import { BrandPanel } from "./BrandPanel";

/** `<AuthFrame><SignInForm /></AuthFrame>` */
export function AuthFrame({ children }: { readonly children: ReactNode }) {
  return (
    <Box component="main" sx={{ minBlockSize: "100dvh", display: "grid", gridTemplateColumns: { xs: "1fr", md: "minmax(0, 1.15fr) minmax(420px, 1fr)" } }}>
      <BrandPanel />
      <Box sx={{ position: "relative", display: "grid", placeItems: "center", p: { xs: 3, sm: 6 }, bgcolor: "background.paper" }}>
        <Box sx={{ position: "absolute", top: 16, insetInlineEnd: 16, color: "text.secondary" }}>
          <SchemeMenu />
        </Box>
        <Box sx={{ inlineSize: "100%", maxInlineSize: 400 }}>{children}</Box>
      </Box>
    </Box>
  );
}
