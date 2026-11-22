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

//! Colours: `#rrggbb` values, the contrast WCAG 2.2 measures between two of them, and the OKLCH space in
//! which a tonal palette keeps the hue of its seed while its lightness changes.

use std::fmt;

/// A colour of sRGB, eight bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub [u8; 3]);

/// White, the light text of a dark background.
pub const WHITE: Rgb = Rgb([255, 255, 255]);

impl Rgb {
    /// The colour written as `#rrggbb`, in either case; none for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let hex = text.strip_prefix('#')?;
        if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).ok();
        Some(Self([channel(0)?, channel(2)?, channel(4)?]))
    }

    /// The relative luminance, as WCAG 2.2 defines it.
    #[must_use]
    pub fn luminance(self) -> f64 {
        let [red, green, blue] = self.0.map(|c| linear(f64::from(c) / 255.0));
        0.0722_f64.mul_add(blue, 0.2126_f64.mul_add(red, 0.7152 * green))
    }

    /// The contrast with another colour, from 1 to 21.
    #[must_use]
    pub fn contrast(self, other: Self) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// The colour halfway to another, channel by channel, rounding halves up.
    #[must_use]
    pub fn halfway(self, other: Self) -> Self {
        let [a, b] = [self.0, other.0];
        Self(
            [0, 1, 2]
                .map(|i| u8::try_from((u16::from(a[i]) + u16::from(b[i])).div_ceil(2)).unwrap_or(u8::MAX)),
        )
    }

    /// The colour in OKLCH: lightness from 0 to 1, chroma, and hue in degrees.
    #[must_use]
    pub fn oklch(self) -> Lch {
        let [red, green, blue] = self.0.map(|c| linear(f64::from(c) / 255.0));
        let long = 0.051_445_992_9_f64
            .mul_add(blue, 0.412_221_470_8_f64.mul_add(red, 0.536_332_536_3 * green))
            .cbrt();
        let medium = 0.107_396_956_6_f64
            .mul_add(blue, 0.211_903_498_2_f64.mul_add(red, 0.680_699_545_1 * green))
            .cbrt();
        let short = 0.629_978_700_5_f64
            .mul_add(blue, 0.088_302_461_9_f64.mul_add(red, 0.281_718_837_6 * green))
            .cbrt();
        let lightness =
            0.004_072_046_8_f64.mul_add(-short, 0.210_454_255_3_f64.mul_add(long, 0.793_617_785 * medium));
        let green_red =
            0.450_593_709_9_f64.mul_add(short, 1.977_998_495_1_f64.mul_add(long, -2.428_592_205 * medium));
        let blue_yellow = 0.808_675_766_f64.mul_add(
            -short,
            0.025_904_037_1_f64.mul_add(long, 0.782_771_766_2 * medium),
        );
        Lch {
            lightness,
            chroma: green_red.hypot(blue_yellow),
            hue: blue_yellow.atan2(green_red).to_degrees().rem_euclid(360.0),
        }
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [r, g, b] = self.0;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

/// A colour in OKLCH.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lch {
    /// From 0, black, to 1, white.
    pub lightness: f64,
    /// From 0, grey, upwards.
    pub chroma: f64,
    /// In degrees.
    pub hue: f64,
}

impl Lch {
    /// The colour in sRGB, with the chroma lowered as little as needed for it to exist there.
    #[must_use]
    pub fn rgb(self) -> Rgb {
        if let Some(rgb) = self.exact() {
            return rgb;
        }
        let (mut low, mut high) = (0.0, self.chroma);
        for _ in 0..24 {
            let middle = f64::midpoint(low, high);
            if (Self {
                chroma: middle,
                ..self
            })
            .exact()
            .is_some()
            {
                low = middle;
            } else {
                high = middle;
            }
        }
        Self { chroma: low, ..self }.exact().unwrap_or(WHITE)
    }

    /// The colour in sRGB when it exists there.
    fn exact(self) -> Option<Rgb> {
        let (green_red, blue_yellow) = (
            self.chroma * self.hue.to_radians().cos(),
            self.chroma * self.hue.to_radians().sin(),
        );
        let long = 0.215_803_757_3_f64
            .mul_add(
                blue_yellow,
                0.396_337_777_4_f64.mul_add(green_red, self.lightness),
            )
            .powi(3);
        let medium = 0.063_854_172_8_f64
            .mul_add(
                -blue_yellow,
                0.105_561_345_8_f64.mul_add(-green_red, self.lightness),
            )
            .powi(3);
        let short = 1.291_485_548_f64
            .mul_add(
                -blue_yellow,
                0.089_484_177_5_f64.mul_add(-green_red, self.lightness),
            )
            .powi(3);
        let red = 0.230_969_929_2_f64.mul_add(
            short,
            4.076_741_662_1_f64.mul_add(long, -3.307_711_591_3 * medium),
        );
        let green = 0.341_319_396_5_f64.mul_add(
            -short,
            (-1.268_438_004_6_f64).mul_add(long, 2.609_757_401_1 * medium),
        );
        let blue = 1.707_614_701_f64.mul_add(
            short,
            (-0.004_196_086_3_f64).mul_add(long, -0.703_418_614_7 * medium),
        );
        let channels = [red, green, blue];
        if channels.iter().any(|c| !(-1e-4..=1.0001).contains(c)) {
            return None;
        }
        Some(Rgb(channels.map(|c| {
            let value = (gamma(c.clamp(0.0, 1.0)) * 255.0).round();
            // The value is within 0 and 255 after the clamp and the rounding.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let byte = value as u8;
            byte
        })))
    }
}

/// A channel of sRGB, from 0 to 1, as linear light.
fn linear(c: f64) -> f64 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear light, from 0 to 1, as a channel of sRGB.
fn gamma(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055_f64.mul_add(c.powf(1.0 / 2.4), -0.055)
    }
}
