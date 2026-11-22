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

/** What a place shows when it has nothing: an icon, why it is empty and, when there is one, what to do. */

import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import type { ReactNode } from "react";

import { Icon } from "@kampus/ui";

/** The props of `EmptyState`. */
export interface EmptyStateProps {
  /** The icon of what is missing. */
  readonly icon: string;
  /** Why it is empty. */
  readonly message: string;
  /** What to do, as a button. */
  readonly action?: ReactNode;
  /** A short line for a small card, instead of the centred block of a page. */
  readonly compact?: boolean;
}

/** `<EmptyState icon="school" message="There are no courses to show." />` */
export function EmptyState({ icon, message, action, compact = false }: EmptyStateProps) {
  if (compact) {
    return (
      <Box sx={{ display: "flex", alignItems: "center", gap: 1.5, py: 1.5, color: "text.secondary" }}>
        <Icon name={icon} size="small" />
        <Typography variant="body2">{message}</Typography>
      </Box>
    );
  }
  return (
    <Box sx={{ display: "grid", justifyItems: "center", gap: 2, py: 6, px: 2, textAlign: "center", color: "text.secondary" }}>
      <Box sx={{ display: "grid", placeItems: "center", inlineSize: 64, blockSize: 64, borderRadius: "50%", bgcolor: "secondary.container", color: "secondary.onContainer" }}>
        <Icon name={icon} size="large" />
      </Box>
      <Typography variant="body1">{message}</Typography>
      {action}
    </Box>
  );
}
