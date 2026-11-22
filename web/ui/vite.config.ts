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

import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

export default defineConfig({
  optimizeDeps: {
    include: [
      "@mui/material/Badge",
      "@mui/material/Box",
      "@mui/material/Button",
      "@mui/material/ButtonGroup",
      "@mui/material/Chip",
      "@mui/material/CircularProgress",
      "@mui/material/CssBaseline",
      "@mui/material/Dialog",
      "@mui/material/DialogActions",
      "@mui/material/DialogContent",
      "@mui/material/DialogTitle",
      "@mui/material/FormControl",
      "@mui/material/FormHelperText",
      "@mui/material/FormLabel",
      "@mui/material/IconButton",
      "@mui/material/InputAdornment",
      "@mui/material/NativeSelect",
      "@mui/material/OutlinedInput",
      "@mui/material/Popover",
      "@mui/material/Snackbar",
      "@mui/material/Stack",
      "@mui/material/ToggleButton",
      "@mui/material/ToggleButtonGroup",
      "@mui/material/Tooltip",
      "@mui/material/Typography",
      "@mui/material/styles",
      "axe-core",
      "react",
      "react-dom",
      "react/jsx-dev-runtime",
      "vitest-browser-react",
    ],
  },
  test: {
    include: ["tests/**/*.test.{ts,tsx}"],
    browser: {
      enabled: true,
      headless: true,
      provider: playwright(),
      instances: [{ browser: "chromium" }],
    },
  },
});
