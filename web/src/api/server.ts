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


/**
 * The API as the server of the web client asks it, on behalf of the person whose request it renders: their session
 * cookie goes with every call, and nothing is cached between people.
 */

import { cookies } from "next/headers";

import { ApiError, answer } from "./problem";

/** Where the server reaches the API. */
const base = process.env["KAMPUS_API_URL"] ?? "http://localhost:8100";

/** Asks the API for a resource, as the person of the request. */
export async function serverApi<T>(path: string): Promise<T> {
  const jar = await cookies();
  const cookie = jar
    .getAll()
    .map((c) => `${c.name}=${c.value}`)
    .join("; ");
  const response = await fetch(`${base}/api/v1${path}`, {
    headers: { Accept: "application/json", ...(cookie ? { Cookie: cookie } : {}) },
    cache: "no-store",
  });
  return answer<T>(response);
}

/** Asks for a part the person may not see, or that this installation does not offer: missing, it is the fallback. */
export async function optional<T>(path: string, fallback: T): Promise<T> {
  try {
    return await serverApi<T>(path);
  } catch (failure) {
    if (failure instanceof ApiError && failure.status < 500) return fallback;
    throw failure;
  }
}
