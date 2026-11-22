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
// @package   kampus-customise
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! Tonal palettes: eleven tones of one hue, from almost white to almost black, from which the colour roles
//! of a theme are taken. Didactika's own palettes are kept as they are; any other seed colour grows a palette
//! in OKLCH that keeps its hue, and the seed itself takes the place of the tone nearest to it.

use super::colour::{Lch, Rgb};

/// The tones of a palette, lightest first.
pub const STEPS: [u16; 11] = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

/// The lightness in OKLCH of each tone of a colour.
const COLOUR_LIGHTNESS: [f64; 11] = [
    0.984, 0.953, 0.91, 0.855, 0.777, 0.704, 0.6, 0.511, 0.437, 0.386, 0.277,
];

/// The lightness in OKLCH of each tone of a neutral, darker from the middle down, as surfaces and their text
/// need.
const NEUTRAL_LIGHTNESS: [f64; 11] = [
    0.984, 0.968, 0.929, 0.869, 0.704, 0.554, 0.446, 0.372, 0.279, 0.208, 0.129,
];

/// The chroma of each tone, relative to the most colourful one.
const CHROMA: [f64; 11] = [0.1, 0.35, 0.65, 0.87, 1.0, 0.95, 0.8, 0.65, 0.53, 0.43, 0.31];

/// The most chroma a palette asks for; what sRGB cannot show is lowered further.
const MAX_CHROMA: f64 = 0.37;

/// Eleven tones of one hue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette(pub [Rgb; 11]);

/// A palette written as eleven `#rrggbb` constants.
const fn palette(tones: [[u8; 3]; 11]) -> Palette {
    let mut out = [Rgb([0, 0, 0]); 11];
    let mut i = 0;
    while i < 11 {
        out[i] = Rgb(tones[i]);
        i += 1;
    }
    Palette(out)
}

/// Didactika's teal, the palette of its primary colour.
pub const TEAL: Palette = palette([
    [0xf0, 0xfd, 0xfa],
    [0xcc, 0xfb, 0xf1],
    [0x99, 0xf6, 0xe4],
    [0x5e, 0xea, 0xd4],
    [0x2d, 0xd4, 0xbf],
    [0x14, 0xb8, 0xa6],
    [0x0d, 0x94, 0x88],
    [0x0f, 0x76, 0x6e],
    [0x11, 0x5e, 0x59],
    [0x13, 0x4e, 0x4a],
    [0x04, 0x2f, 0x2e],
]);

/// Didactika's slate, the palette of its secondary and neutral colours.
pub const SLATE: Palette = palette([
    [0xf8, 0xfa, 0xfc],
    [0xf1, 0xf5, 0xf9],
    [0xe2, 0xe8, 0xf0],
    [0xcb, 0xd5, 0xe1],
    [0x94, 0xa3, 0xb8],
    [0x64, 0x74, 0x8b],
    [0x47, 0x55, 0x69],
    [0x33, 0x41, 0x55],
    [0x1e, 0x29, 0x3b],
    [0x0f, 0x17, 0x2a],
    [0x02, 0x06, 0x17],
]);

/// Didactika's cyan, the palette of its accent.
pub const CYAN: Palette = palette([
    [0xec, 0xfe, 0xff],
    [0xcf, 0xfa, 0xfe],
    [0xa5, 0xf3, 0xfc],
    [0x67, 0xe8, 0xf9],
    [0x22, 0xd3, 0xee],
    [0x06, 0xb6, 0xd4],
    [0x08, 0x91, 0xb2],
    [0x0e, 0x74, 0x90],
    [0x15, 0x5e, 0x75],
    [0x16, 0x4e, 0x63],
    [0x08, 0x33, 0x44],
]);

/// The red of what removes or fails.
pub const RED: Palette = palette([
    [0xfe, 0xf2, 0xf2],
    [0xfe, 0xe2, 0xe2],
    [0xfe, 0xca, 0xca],
    [0xfc, 0xa5, 0xa5],
    [0xf8, 0x71, 0x71],
    [0xef, 0x44, 0x44],
    [0xdc, 0x26, 0x26],
    [0xb9, 0x1c, 0x1c],
    [0x99, 0x1b, 0x1b],
    [0x7f, 0x1d, 0x1d],
    [0x45, 0x0a, 0x0a],
]);

/// The green of what is done or safe.
pub const GREEN: Palette = palette([
    [0xf0, 0xfd, 0xf4],
    [0xdc, 0xfc, 0xe7],
    [0xbb, 0xf7, 0xd0],
    [0x86, 0xef, 0xac],
    [0x4a, 0xde, 0x80],
    [0x22, 0xc5, 0x5e],
    [0x16, 0xa3, 0x4a],
    [0x15, 0x80, 0x3d],
    [0x16, 0x65, 0x34],
    [0x14, 0x53, 0x2d],
    [0x05, 0x2e, 0x16],
]);

/// The amber of what needs attention.
pub const AMBER: Palette = palette([
    [0xff, 0xfb, 0xeb],
    [0xfe, 0xf3, 0xc7],
    [0xfd, 0xe6, 0x8a],
    [0xfc, 0xd3, 0x4d],
    [0xfb, 0xbf, 0x24],
    [0xf5, 0x9e, 0x0b],
    [0xd9, 0x77, 0x06],
    [0xb4, 0x53, 0x09],
    [0x92, 0x40, 0x0e],
    [0x78, 0x35, 0x0f],
    [0x45, 0x1a, 0x03],
]);

/// The sky blue of what informs.
pub const SKY: Palette = palette([
    [0xf0, 0xf9, 0xff],
    [0xe0, 0xf2, 0xfe],
    [0xba, 0xe6, 0xfd],
    [0x7d, 0xd3, 0xfc],
    [0x38, 0xbd, 0xf8],
    [0x0e, 0xa5, 0xe9],
    [0x02, 0x84, 0xc7],
    [0x03, 0x69, 0xa1],
    [0x07, 0x59, 0x85],
    [0x0c, 0x4a, 0x6e],
    [0x08, 0x2f, 0x49],
]);

impl Palette {
    /// One tone, by its step (`50`, `100`… `950`); the nearest darker one for a step between two.
    #[must_use]
    pub fn tone(&self, step: u16) -> Rgb {
        let at = STEPS.iter().position(|s| *s >= step).unwrap_or(STEPS.len() - 1);
        self.0[at]
    }

    /// The palette grown from a seed colour: its hue at every lightness of a colour, or of a neutral, with
    /// the seed in the place of the tone nearest to it, so the colour chosen appears exactly.
    #[must_use]
    pub fn grown(seed: Rgb, neutral: bool) -> Self {
        let lightness = if neutral {
            NEUTRAL_LIGHTNESS
        } else {
            COLOUR_LIGHTNESS
        };
        let lch = seed.oklch();
        let nearest = lightness
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| (*a - lch.lightness).abs().total_cmp(&(*b - lch.lightness).abs()))
            .map_or(0, |(at, _)| at);
        let peak = (lch.chroma / CHROMA[nearest]).min(MAX_CHROMA);
        let mut tones = [seed; 11];
        for (at, tone) in tones.iter_mut().enumerate() {
            if at != nearest {
                *tone = Lch {
                    lightness: lightness[at],
                    chroma: peak * CHROMA[at],
                    hue: lch.hue,
                }
                .rgb();
            }
        }
        Self(tones)
    }

    /// The palette of a seed colour: one of Didactika's when the seed is a tone of it, grown from the seed
    /// otherwise.
    #[must_use]
    pub fn of(seed: Rgb) -> Self {
        [TEAL, CYAN, RED, GREEN, AMBER, SKY, SLATE]
            .into_iter()
            .find(|p| p.0.contains(&seed))
            .unwrap_or_else(|| Self::grown(seed, false))
    }

    /// The palette of a neutral seed: Didactika's slate when the seed is a tone of it, grown from the seed
    /// otherwise.
    #[must_use]
    pub fn neutral(seed: Rgb) -> Self {
        if SLATE.0.contains(&seed) {
            SLATE
        } else {
            Self::grown(seed, true)
        }
    }
}
