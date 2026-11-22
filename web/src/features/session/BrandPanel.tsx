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

/**
 * The side of the sign-in page that shows what Kampus is: the gradient of the brand, a headline, what it offers, and
 * a glimpse of the product (a course with its progress, a figure) floating over it, all in the colours of the theme
 * of the campus.
 */

import Box from "@mui/material/Box";
import LinearProgress from "@mui/material/LinearProgress";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { keyframes } from "@mui/material/styles";

import { CoverArt, Icon } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";

const float = keyframes`
  0%, 100% { transform: translateY(0) rotate(var(--tilt)); }
  50% { transform: translateY(-10px) rotate(var(--tilt)); }
`;

/** What the panel lists, with its icon. */
const FEATURES = [
  ["palette", "signin.feature.design"],
  ["verified", "signin.feature.access"],
  ["widgets", "signin.feature.modules"],
] as const;

/** A card of the glimpse of the product, as glass over the gradient. */
const glass = {
  bgcolor: "color-mix(in srgb, var(--k-palette-background-paper) 92%, transparent)",
  color: "text.primary",
  borderRadius: 4,
  boxShadow: "0 24px 56px -12px color-mix(in srgb, var(--k-palette-surface-scrim) 45%, transparent)",
  backdropFilter: "blur(8px)",
  animation: `${float} 7s ease-in-out infinite`,
  "@media (prefers-reduced-motion: reduce)": { animation: "none" },
} as const;

/** `<BrandPanel />` */
export function BrandPanel() {
  const { t } = useI18n();
  return (
    <Box
      sx={{
        position: "relative",
        overflow: "hidden",
        color: "brand.contrastText",
        backgroundImage:
          "radial-gradient(circle at 85% 15%, color-mix(in srgb, var(--k-palette-accent-main) 35%, transparent) 0 18%, transparent 42%), radial-gradient(circle at 10% 90%, color-mix(in srgb, var(--k-palette-brand-contrastText) 12%, transparent) 0 22%, transparent 48%), var(--k-palette-gradient-bar)",
        p: { xs: 3, md: 6 },
        display: "flex",
        flexDirection: "column",
        minBlockSize: { md: "100dvh" },
      }}
    >
      <Box
        aria-hidden
        sx={{
          position: "absolute",
          inset: 0,
          opacity: 0.18,
          backgroundImage: "radial-gradient(currentColor 1px, transparent 1.5px)",
          backgroundSize: "22px 22px",
          maskImage: "linear-gradient(160deg, black, transparent 70%)",
        }}
      />
      <Stack direction="row" spacing={1.5} sx={{ alignItems: "center", position: "relative" }}>
        <Box component="img" src="/kampus-icon.svg" alt="" sx={{ inlineSize: 40, blockSize: 40, borderRadius: 2.5, boxShadow: 4 }} />
        <Typography variant="h3" component="span">
          {t("app.name")}
        </Typography>
      </Stack>

      <Box sx={{ position: "relative", mt: { xs: 3, md: "auto" }, maxInlineSize: 520 }}>
        <Typography variant="h1" component="p" sx={{ fontSize: { xs: "1.75rem", md: "2.75rem" }, lineHeight: 1.08 }}>
          {t("signin.hero.title")}
        </Typography>
        <Typography sx={{ mt: 2, opacity: 0.9, fontSize: { md: "1.0625rem" }, display: { xs: "none", sm: "block" } }}>{t("signin.hero.lead")}</Typography>
        <Stack component="ul" spacing={1.5} sx={{ listStyle: "none", p: 0, mt: 4, mb: 0, display: { xs: "none", md: "flex" } }}>
          {FEATURES.map(([icon, key]) => (
            <Stack component="li" key={key} direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
              <Box sx={{ display: "grid", placeItems: "center", inlineSize: 36, blockSize: 36, borderRadius: 2.5, bgcolor: "color-mix(in srgb, currentColor 14%, transparent)", border: "1px solid color-mix(in srgb, currentColor 22%, transparent)" }}>
                <Icon name={icon} size="small" />
              </Box>
              <Typography variant="subtitle2">{t(key)}</Typography>
            </Stack>
          ))}
        </Stack>
      </Box>

      <Box aria-hidden sx={{ position: "relative", blockSize: 260, mt: 5, mb: "auto", display: { xs: "none", lg: "block" } }}>
        <Box sx={{ ...glass, position: "absolute", insetInlineStart: 0, top: 0, inlineSize: 300, overflow: "hidden", "--tilt": "-3deg" }}>
          <CoverArt seed="kampus-preview" icon="science" height={84} />
          <Box sx={{ p: 2 }}>
            <Typography variant="overline" color="text.secondary">
              {t("signin.preview.code")}
            </Typography>
            <Typography variant="h4" component="span" sx={{ display: "block", mb: 1.5 }}>
              {t("signin.preview.course")}
            </Typography>
            <LinearProgress variant="determinate" value={68} sx={{ blockSize: 8 }} />
          </Box>
        </Box>
        <Box sx={{ ...glass, position: "absolute", insetInlineStart: 260, top: 120, p: 2, inlineSize: 200, "--tilt": "4deg", animationDelay: "1.2s" }}>
          <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
            <Box sx={{ display: "grid", placeItems: "center", inlineSize: 40, blockSize: 40, borderRadius: 2.5, backgroundImage: "var(--k-palette-gradient-primary)", color: "primary.contrastText" }}>
              <Icon name="school" size="small" />
            </Box>
            <Box>
              <Typography variant="h3" component="span" sx={{ display: "block", lineHeight: 1 }}>
                36
              </Typography>
              <Typography variant="caption" color="text.secondary">
                {t("stats.courses")}
              </Typography>
            </Box>
          </Stack>
        </Box>
      </Box>
    </Box>
  );
}
