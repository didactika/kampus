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


/**
 * The design system of Kampus: React components on MUI with Didactika's look and meaning, the MUI theme of a scope,
 * the tokens of Didactika's theme, and icons by name that a theme can override.
 */

export * from "./components/Button/index";
export * from "./components/ButtonGroup/index";
export * from "./components/ConfirmDialog/index";
export * from "./components/Icon/index";
export * from "./components/Loading/index";
export * from "./components/SelectField/index";
export * from "./components/StatusBadge/index";
export * from "./components/TextField/index";
export * from "./components/Toasts/index";
export * from "./patterns/CoverArt";
export * from "./patterns/FilterGroup";
export * from "./patterns/ListToolbar";
export * from "./patterns/Pager";
export * from "./patterns/PillSelect";
export * from "./patterns/ProgressRing";
export * from "./patterns/ViewToggle";
export { CORE_ICONS } from "./icons/core";
export { defineIcons, iconNames, iconOf, onIconsChange, overrideIcons, type IconSource } from "./icons/registry";
export type { SurfaceRoles } from "./theme/augment";
export { createKampusTheme, paletteOf } from "./theme/create-theme";
export { KampusThemeProvider, type KampusThemeProviderProps } from "./theme/KampusThemeProvider";
export { DIDACTIKA, DIDACTIKA_TOKENS, type Roles, type ScopeTheme, type ThemeTokens } from "./theme/scope-theme";
export { contrast, luminance } from "./tokens/contrast";
export { flatten, paths, roles, type Token, type TokenNode } from "./tokens/tokens";
