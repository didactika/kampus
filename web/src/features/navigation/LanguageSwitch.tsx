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
 * The language of the person, chosen from the bar among the languages installed. It is kept in their profile, so the
 * campus speaks it on every device, and the page is drawn again in it at once.
 */

import { useRouter } from "next/navigation";

import { useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { Profile } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { LANGUAGES } from "../../i18n/translator";
import { LanguagePicker } from "../editor/LanguagePicker";

/** The props of `LanguageSwitch`. */
export interface LanguageSwitchProps {
  readonly profile: Profile | null;
  /** Whether it sits on the bar of the campus or on a surface, as in the drawer of a phone. */
  readonly tone?: "bar" | "surface";
}

/** `<LanguageSwitch profile={profile} tone="bar" />` */
export function LanguageSwitch({ profile, tone = "bar" }: LanguageSwitchProps) {
  const { t, language } = useI18n();
  const router = useRouter();
  const toast = useToast();
  if (!profile || LANGUAGES.length < 2) return null;
  const choose = async (code: string) => {
    if (code === language) return;
    try {
      await browserApi<Profile>("/users/me/profile", {
        method: "PUT",
        headers: { "If-Match": `"${profile.version}"` },
        body: {
          given_name: profile.given_name,
          family_name: profile.family_name,
          display_name: profile.display_name,
          language_code: code,
          time_zone: profile.time_zone,
        },
      });
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    }
  };
  return (
    <LanguagePicker
      compact={tone === "bar"}
      tone={tone}
      label={(name) => t("nav.language", { language: name })}
      languages={LANGUAGES}
      value={language}
      onChange={(code) => void choose(code)}
    />
  );
}
