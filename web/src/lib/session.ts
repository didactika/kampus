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

import { type Me, api } from "./api";

let current: Me | null = null;

/** The signed-in account, or none; the answer is asked of the API once and kept until it changes. */
export async function me(refresh = false): Promise<Me | null> {
  if (current && !refresh) return current;
  try {
    current = await api<Me>("/identity/me");
  } catch {
    current = null;
  }
  return current;
}

/** Signs in; the session cookie is set by the API. */
export async function signIn(name: string, password: string): Promise<Me | null> {
  await api("/identity/auth/sign-in", { method: "POST", body: { name, password } });
  return me(true);
}

/** Signs out and forgets the account. */
export async function signOut(): Promise<void> {
  try {
    await api("/identity/auth/sign-out", { method: "POST" });
  } finally {
    current = null;
  }
}
