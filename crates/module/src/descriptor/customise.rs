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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! What a module contributes to customisation: the words of its domain, which a campus may rename; the
//! feature switches it offers; the kinds of entity that accept custom fields; and the pages, widgets and
//! menus a campus arranges.

/// The forms of a word in one language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermForms {
    /// The language, a BCP 47 tag (`en`, `es`).
    pub language: &'static str,
    /// The word for one.
    pub singular: &'static str,
    /// The word for several.
    pub plural: &'static str,
}

/// A word of a module's domain, such as course or teacher, that a campus may rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermDeclaration {
    /// Its code, `<module>.<word>` (`courses.course`).
    pub code: &'static str,
    /// Its forms in each language the module provides; the first is used when a language has none.
    pub forms: &'static [TermForms],
}

/// A feature a module offers that a scope may turn off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlagDeclaration {
    /// Its code, `<module>.<feature>`.
    pub code: &'static str,
    /// What it turns on.
    pub summary: &'static str,
    /// Whether it is on where no scope sets it.
    pub default: bool,
}

/// A kind of entity of a module that accepts custom fields, such as a course.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldKindDeclaration {
    /// Its code, `<module>.<entity>` (`courses.course`).
    pub code: &'static str,
    /// What it is.
    pub summary: &'static str,
}

/// A kind of page a module shows, with the regions a layout places widgets in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageKindDeclaration {
    /// `<module>.<page>`, unique in the build.
    pub code: &'static str,
    /// What the page is, for people who arrange it.
    pub summary: &'static str,
    /// Its regions, in the order they are shown.
    pub regions: &'static [&'static str],
}

/// A setting of a widget, with the kind of value it takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetSetting {
    /// Its key, unique in the widget.
    pub key: &'static str,
    /// The kind of value, with its default.
    pub kind: crate::ValueKind,
}

/// A widget a module offers for layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetDeclaration {
    /// `<module>.<widget>`, unique in the build.
    pub code: &'static str,
    /// What it shows.
    pub summary: &'static str,
    /// The page kinds it may be placed in; every page kind when empty.
    pub page_kinds: &'static [&'static str],
    /// Its settings; a placement sets only these, each of its kind.
    pub settings: &'static [WidgetSetting],
}

/// A kind of navigation menu a module shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuKindDeclaration {
    /// `<module>.<menu>`, unique in the build.
    pub code: &'static str,
    /// Where it is shown.
    pub summary: &'static str,
}
