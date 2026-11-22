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
 * The design system of Kampus: Web Components that follow the patterns of Material 3, themed only by design
 * tokens and usable from any framework. Importing this module defines every element; the tokens of the default theme
 * come from `@kampus/ui/tokens.css`, and the theme of a scope, which the server serves as the same custom
 * properties, replaces them. Icons are Material Symbols by name, and a theme can override any of them.
 */

export { KAlert, type Tone } from "./components/alert/alert.js";
export { KBadge } from "./components/badge/badge.js";
export { KButton, type ButtonTone, type ButtonVariant } from "./components/button/button.js";
export { KCard, type CardVariant } from "./components/card/card.js";
export { KCheckbox } from "./components/checkbox/checkbox.js";
export { KDialog } from "./components/dialog/dialog.js";
export { KIcon } from "./components/icon/icon.js";
export { KInput, type InputType } from "./components/input/input.js";
export { KSelect, type SelectOption } from "./components/select/select.js";
export { KSpinner } from "./components/spinner/spinner.js";
export { KTabPanel, KTabs } from "./components/tabs/tabs.js";
export { KTextarea } from "./components/textarea/textarea.js";
export { KToasts, toast } from "./components/toast/toast.js";
export { FormControl } from "./forms/form-control.js";
export { CORE_ICONS } from "./icons/core.js";
export { defineIcons, iconNames, iconOf, onIconsChange, overrideIcons, type IconSource } from "./icons/registry.js";
export { type TypeRole } from "./styles/typography.js";
export { contrast, luminance } from "./tokens/contrast.js";
export {
  declarations,
  flatten,
  paths,
  property,
  roles,
  stylesheet,
  type ThemeDocuments,
  type Token,
  type TokenNode,
} from "./tokens/tokens.js";
