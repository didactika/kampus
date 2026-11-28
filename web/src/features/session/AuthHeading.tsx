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

/** The head of a screen of access (signing in, a second factor, the outcome of a link): its title and what it asks. */

import Typography from "@mui/material/Typography";
import type { ReactNode } from "react";

/** The props of `AuthHeading`. */
export interface AuthHeadingProps {
  readonly title: ReactNode;
  /** What the screen asks of the person. */
  readonly lead?: ReactNode;
}

/** `<AuthHeading title={t("signin.welcome")} lead={t("signin.lead")} />` */
export function AuthHeading({ title, lead }: AuthHeadingProps) {
  return (
    <>
      <Typography variant="h1" sx={{ fontSize: { xs: "1.75rem", md: "2rem" } }}>
        {title}
      </Typography>
      <Typography color="text.secondary" sx={{ mt: 1, mb: 4, minBlockSize: lead ? undefined : 0 }}>
        {lead}
      </Typography>
    </>
  );
}
