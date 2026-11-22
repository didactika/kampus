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
 * The icons the design system uses, from Material Symbols (rounded, weight 400), named as there. Each meaning has one
 * icon everywhere: `delete` removes, `edit` changes, `visibility` and `visibility_off` show or hide, `info` opens
 * details, `history` opens versions.
 */

import iconAdd from "@material-symbols/svg-400/rounded/add.svg?raw";
import iconArrowBack from "@material-symbols/svg-400/rounded/arrow_back.svg?raw";
import iconCalendarMonth from "@material-symbols/svg-400/rounded/calendar_month.svg?raw";
import iconCheck from "@material-symbols/svg-400/rounded/check.svg?raw";
import iconCheckCircle from "@material-symbols/svg-400/rounded/check_circle.svg?raw";
import iconChevronLeft from "@material-symbols/svg-400/rounded/chevron_left.svg?raw";
import iconChevronRight from "@material-symbols/svg-400/rounded/chevron_right.svg?raw";
import iconClose from "@material-symbols/svg-400/rounded/close.svg?raw";
import iconContentCopy from "@material-symbols/svg-400/rounded/content_copy.svg?raw";
import iconDarkMode from "@material-symbols/svg-400/rounded/dark_mode.svg?raw";
import iconDelete from "@material-symbols/svg-400/rounded/delete.svg?raw";
import iconDownload from "@material-symbols/svg-400/rounded/download.svg?raw";
import iconEdit from "@material-symbols/svg-400/rounded/edit.svg?raw";
import iconError from "@material-symbols/svg-400/rounded/error.svg?raw";
import iconFilterList from "@material-symbols/svg-400/rounded/filter_list.svg?raw";
import iconHelp from "@material-symbols/svg-400/rounded/help.svg?raw";
import iconHistory from "@material-symbols/svg-400/rounded/history.svg?raw";
import iconHome from "@material-symbols/svg-400/rounded/home.svg?raw";
import iconInfo from "@material-symbols/svg-400/rounded/info.svg?raw";
import iconKeyboardArrowDown from "@material-symbols/svg-400/rounded/keyboard_arrow_down.svg?raw";
import iconKeyboardArrowUp from "@material-symbols/svg-400/rounded/keyboard_arrow_up.svg?raw";
import iconLanguage from "@material-symbols/svg-400/rounded/language.svg?raw";
import iconLightMode from "@material-symbols/svg-400/rounded/light_mode.svg?raw";
import iconLogout from "@material-symbols/svg-400/rounded/logout.svg?raw";
import iconMail from "@material-symbols/svg-400/rounded/mail.svg?raw";
import iconMenu from "@material-symbols/svg-400/rounded/menu.svg?raw";
import iconMoreVert from "@material-symbols/svg-400/rounded/more_vert.svg?raw";
import iconNotifications from "@material-symbols/svg-400/rounded/notifications.svg?raw";
import iconOpenInNew from "@material-symbols/svg-400/rounded/open_in_new.svg?raw";
import iconPerson from "@material-symbols/svg-400/rounded/person.svg?raw";
import iconRefresh from "@material-symbols/svg-400/rounded/refresh.svg?raw";
import iconSchool from "@material-symbols/svg-400/rounded/school.svg?raw";
import iconSearch from "@material-symbols/svg-400/rounded/search.svg?raw";
import iconSettings from "@material-symbols/svg-400/rounded/settings.svg?raw";
import iconUpload from "@material-symbols/svg-400/rounded/upload.svg?raw";
import iconVisibility from "@material-symbols/svg-400/rounded/visibility.svg?raw";
import iconVisibilityOff from "@material-symbols/svg-400/rounded/visibility_off.svg?raw";
import iconWarning from "@material-symbols/svg-400/rounded/warning.svg?raw";

import { defineIcons } from "./registry.js";

/** The icons of the design system, by name. */
export const CORE_ICONS: Readonly<Record<string, string>> = {
  add: iconAdd,
  arrow_back: iconArrowBack,
  calendar_month: iconCalendarMonth,
  check: iconCheck,
  check_circle: iconCheckCircle,
  chevron_left: iconChevronLeft,
  chevron_right: iconChevronRight,
  close: iconClose,
  content_copy: iconContentCopy,
  dark_mode: iconDarkMode,
  delete: iconDelete,
  download: iconDownload,
  edit: iconEdit,
  error: iconError,
  filter_list: iconFilterList,
  help: iconHelp,
  history: iconHistory,
  home: iconHome,
  info: iconInfo,
  keyboard_arrow_down: iconKeyboardArrowDown,
  keyboard_arrow_up: iconKeyboardArrowUp,
  language: iconLanguage,
  light_mode: iconLightMode,
  logout: iconLogout,
  mail: iconMail,
  menu: iconMenu,
  more_vert: iconMoreVert,
  notifications: iconNotifications,
  open_in_new: iconOpenInNew,
  person: iconPerson,
  refresh: iconRefresh,
  school: iconSchool,
  search: iconSearch,
  settings: iconSettings,
  upload: iconUpload,
  visibility: iconVisibility,
  visibility_off: iconVisibilityOff,
  warning: iconWarning,
};

defineIcons(CORE_ICONS);
