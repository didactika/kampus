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

/// The height of a row of the grid of a page, from the heights everyone knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowHeight {
    /// 48 pixels: lists and short texts.
    Compact,
    /// 72 pixels: most pages.
    Regular,
    /// 96 pixels: pages of large figures and pictures.
    Tall,
}

impl RowHeight {
    /// Its code, as stored and sent.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Regular => "regular",
            Self::Tall => "tall",
        }
    }

    /// Its height in CSS pixels.
    #[must_use]
    pub const fn pixels(self) -> u16 {
        match self {
            Self::Compact => 48,
            Self::Regular => 72,
            Self::Tall => 96,
        }
    }
}

/// The grid a page lays its widgets on: columns of equal width and rows of one height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    /// How many columns it has.
    pub columns: u8,
    /// How high a row is.
    pub row: RowHeight,
}

impl Grid {
    /// Twelve columns of regular rows, the grid of most pages.
    pub const STANDARD: Self = Self {
        columns: 12,
        row: RowHeight::Regular,
    };
}

/// A size on a grid, in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cells {
    /// Its width, in columns.
    pub w: u8,
    /// Its height, in rows.
    pub h: u8,
}

impl Cells {
    /// A size of `w` columns by `h` rows.
    #[must_use]
    pub const fn new(w: u8, h: u8) -> Self {
        Self { w, h }
    }
}

/// How a person may change the size of a widget on a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resize {
    /// It keeps its size.
    None,
    /// Only its width changes.
    Horizontal,
    /// Only its height changes.
    Vertical,
    /// Both change.
    Both,
}

impl Resize {
    /// Its code, as stored and sent.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
            Self::Both => "both",
        }
    }
}

/// The sizes a widget takes on one kind of page: the smallest, the one it starts with and the largest, and how a
/// person may change it between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetSize {
    /// The page kind, `<module>.<page>`.
    pub page_kind: &'static str,
    /// The smallest size.
    pub min: Cells,
    /// The size it takes when it is placed.
    pub default: Cells,
    /// The largest size.
    pub max: Cells,
    /// How it may be resized.
    pub resize: Resize,
}

impl WidgetSize {
    /// One size it always keeps, on a page kind: what it shows reads the same however the page is arranged.
    #[must_use]
    pub const fn fixed(page_kind: &'static str, size: Cells) -> Self {
        Self {
            page_kind,
            min: size,
            default: size,
            max: size,
            resize: Resize::None,
        }
    }

    /// What is wrong with it on a grid; nothing when it fits.
    #[must_use]
    pub fn problem(&self, grid: Grid) -> Option<&'static str> {
        let Self {
            min,
            default,
            max,
            resize,
            ..
        } = *self;
        if min.w == 0 || min.h == 0 {
            return Some("a size has no width or no height");
        }
        if !(min.w <= default.w && default.w <= max.w && min.h <= default.h && default.h <= max.h) {
            return Some("the default size is not between the smallest and the largest");
        }
        if max.w > grid.columns {
            return Some("the largest size is wider than the grid");
        }
        let (wide, high) = (min.w != max.w, min.h != max.h);
        let allowed = match resize {
            Resize::None => !wide && !high,
            Resize::Horizontal => !high,
            Resize::Vertical => !wide,
            Resize::Both => true,
        };
        if allowed {
            None
        } else {
            Some("the sizes change in a direction the widget may not be resized in")
        }
    }
}

/// A kind of page a module shows, laid out as a grid of widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageKindDeclaration {
    /// `<module>.<page>`, unique in the build.
    pub code: &'static str,
    /// What the page is, for people who arrange it.
    pub summary: &'static str,
    /// The grid its widgets are placed on.
    pub grid: Grid,
    /// Whether each person arranges it for themselves, in their own scope, such as the home: what they change
    /// is theirs alone. A page that is not personal is arranged for everyone who sees it, such as a course.
    pub personal: bool,
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
    /// The sizes it takes on each page kind it may be placed on; it is placed on no other.
    pub sizes: &'static [WidgetSize],
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
