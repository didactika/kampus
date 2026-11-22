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
 * The document of every page: its language and direction, the fonts of the theme served by Kampus itself, the
 * scheme chosen before the first paint, and the theme of the campus of the person.
 */

import "@fontsource-variable/inter/index.css";

import InitColorSchemeScript from "@mui/material/InitColorSchemeScript";
import { AppRouterCacheProvider } from "@mui/material-nextjs/v16-appRouter";
import type { Metadata, Viewport } from "next";
import type { ReactNode } from "react";

import { currentAccount } from "../features/session/current-account";
import { Providers } from "../features/shell/Providers";
import { requestI18n } from "../i18n/server";
import { scopeTheme } from "../theme/scope-theme";

/** The title of every page, after the name of the application in the language of the request, and the icon. */
export async function generateMetadata(): Promise<Metadata> {
  const { t } = await requestI18n();
  const name = t("app.name");
  return { title: { default: name, template: `%s · ${name}` }, icons: { icon: "/kampus-icon.svg" } };
}

export const viewport: Viewport = { width: "device-width", initialScale: 1 };

/** The root of every page. */
export default async function RootLayout({ children }: { readonly children: ReactNode }) {
  const { language, direction } = await requestI18n();
  const account = await currentAccount();
  const theme = await scopeTheme(account?.campus);
  return (
    <html lang={language} dir={direction} suppressHydrationWarning>
      <body>
        <InitColorSchemeScript attribute="data-theme" defaultMode={theme.tokens.scheme ?? "system"} modeStorageKey="kampus-scheme" />
        <AppRouterCacheProvider options={{ key: "k" }}>
          <Providers language={language} theme={theme}>
            {children}
          </Providers>
        </AppRouterCacheProvider>
      </body>
    </html>
  );
}
