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
// @package   web/src
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

import { ApiError } from "../lib/api";
import { html } from "../lib/html";
import { t } from "../lib/i18n";
import { navigate, type Context } from "../lib/router";
import { signIn } from "../lib/session";
import type { KField } from "../ui/components";

/** The sign-in page. */
export async function signin({ root }: Context): Promise<void> {
  root.innerHTML = html`
    <h1>${t("signin.title")}</h1>
    <form id="signin" novalidate>
      <k-field label="${t("signin.name")}" name="name" autocomplete="username" required></k-field>
      <k-field label="${t("signin.password")}" name="password" type="password" autocomplete="current-password" required></k-field>
      <p id="signin-error" class="error" role="alert"></p>
      <k-button type="submit">${t("signin.submit")}</k-button>
    </form>
  `.value;
  const form = root.querySelector<HTMLFormElement>("#signin");
  const error = root.querySelector<HTMLElement>("#signin-error");
  form?.addEventListener("submit", (event) => {
    event.preventDefault();
    const [name, password] = Array.from(form.querySelectorAll<KField>("k-field")).map((f) => f.value);
    void (async () => {
      try {
        await signIn(name ?? "", password ?? "");
        navigate("/");
      } catch (failure) {
        if (error) error.textContent = failure instanceof ApiError && failure.status < 500 ? t("signin.failed") : t("error.generic");
      }
    })();
  });
  root.querySelector<KField>("k-field")?.focus();
}
