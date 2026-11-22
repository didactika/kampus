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


"use client";

/**
 * The welcome of the home of a campus: a band in the colours of the brand that greets the person, says what the page
 * is for and shows at a glance how many courses, categories and unread notifications there are.
 */

import Box from "@mui/material/Box";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import Link from "next/link";

import { Icon } from "@kampus/ui";

import { useI18n } from "../../i18n/I18nProvider";

/** One figure of the welcome. */
interface Figure {
  readonly icon: string;
  readonly label: string;
  readonly value: number;
  readonly href?: string;
}

/** The figures of the welcome. */
export interface WelcomeFigures {
  /** How many courses the catalogue has. */
  readonly courses: number;
  /** How many categories it has. */
  readonly categories: number;
  /** How many notifications are unread, when the person may see them. */
  readonly unread: number | null;
}

/** The props of `Welcome`. */
export interface WelcomeProps {
  /** The name to greet. */
  readonly name: string;
  /** A heading of the scope instead of the greeting. */
  readonly heading?: string;
  /** The figures, when the welcome shows them. */
  readonly figures: WelcomeFigures | null;
}

/** `<Welcome name="lea" courses={12} categories={3} unread={2} />` */
export function Welcome({ name, heading, figures: counts }: WelcomeProps) {
  const { t, number } = useI18n();
  const figures: Figure[] = counts
    ? [
        { icon: "school", label: t("stats.courses"), value: counts.courses, href: "/courses" },
        { icon: "category", label: t("stats.categories"), value: counts.categories },
        ...(counts.unread === null ? [] : [{ icon: "notifications", label: t("stats.unread"), value: counts.unread, href: "/notifications" }]),
      ]
    : [];
  return (
    <Box
      component="section"
      aria-labelledby="welcome"
      sx={{
        position: "relative",
        overflow: "hidden",
        p: { xs: 3, md: 5 },
        borderRadius: (theme) => `${theme.shape.extraLarge}px`,
        color: "brand.contrastText",
        background:
          "radial-gradient(circle at 92% -10%, color-mix(in srgb, var(--k-palette-accent-main) 22%, transparent) 0 22%, transparent 46%), radial-gradient(circle at -5% 115%, color-mix(in srgb, var(--k-palette-brand-contrastText) 10%, transparent) 0 28%, transparent 52%), var(--k-palette-gradient-bar)",
        boxShadow: 3,
      }}
    >
      <Box
        aria-hidden
        sx={{
          position: "absolute",
          insetInlineEnd: { xs: -60, md: 40 },
          insetBlockStart: { xs: -40, md: 24 },
          inlineSize: 220,
          blockSize: 220,
          borderRadius: "38% 62% 55% 45% / 45% 40% 60% 55%",
          border: "2px solid color-mix(in srgb, currentColor 25%, transparent)",
          rotate: "12deg",
        }}
      />
      <Typography variant="overline" sx={{ opacity: 0.85, letterSpacing: "0.12em", textTransform: "uppercase" }}>
        Kampus
      </Typography>
      <Typography id="welcome" variant="h1" sx={{ mt: 1, fontSize: { md: "2.75rem" }, lineHeight: 1.15, maxInlineSize: "18ch" }}>
        {heading || t("home.greeting", { name })}
      </Typography>
      <Typography variant="body1" sx={{ mt: 1.5, opacity: 0.9, maxInlineSize: "52ch" }}>
        {t("home.lead")}
      </Typography>
      {figures.length ? (
      <Box sx={{ display: "grid", gridTemplateColumns: { xs: "repeat(3, minmax(0, 1fr))", sm: "repeat(3, minmax(0, 170px))" }, gap: 1.5, mt: 4 }}>
        {figures.map((figure) => {
          const tile = (
            <Stack
              direction={{ xs: "column", sm: "row" }}
              spacing={{ xs: 0.5, sm: 1.5 }}
              sx={{
                alignItems: { xs: "flex-start", sm: "center" },
                px: { xs: 1.5, sm: 2 },
                py: 1.5,
                blockSize: "100%",
                borderRadius: 3,
                bgcolor: "color-mix(in srgb, currentColor 12%, transparent)",
                border: "1px solid color-mix(in srgb, currentColor 18%, transparent)",
                backdropFilter: "blur(6px)",
                transition: "background-color 200ms",
                ...(figure.href ? { "&:hover": { bgcolor: "color-mix(in srgb, currentColor 20%, transparent)" } } : {}),
              }}
            >
              <Icon name={figure.icon} />
              <Box>
                <Typography variant="h3" component="span" sx={{ display: "block", lineHeight: 1.1 }}>
                  {number(figure.value)}
                </Typography>
                <Typography variant="caption" sx={{ opacity: 0.85 }}>
                  {figure.label}
                </Typography>
              </Box>
            </Stack>
          );
          return figure.href ? (
            <Box key={figure.label} component={Link} href={figure.href} sx={{ color: "inherit", textDecoration: "none", borderRadius: 3 }}>
              {tile}
            </Box>
          ) : (
            <Box key={figure.label}>{tile}</Box>
          );
        })}
      </Box>
      ) : null}
    </Box>
  );
}
