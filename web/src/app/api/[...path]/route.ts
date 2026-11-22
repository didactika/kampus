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
 * The API through the origin of the web client: the browser asks `/api/…` here and the request goes on to the API
 * the server is configured with at run time, with its body, its session cookie and the headers the API reads, and
 * the answer comes back as it was, cookies included.
 */

import type { NextRequest } from "next/server";

/** The headers of a request the API reads. */
const FORWARDED = ["accept", "accept-language", "content-type", "cookie", "if-match", "if-none-match", "idempotency-key", "x-kampus", "origin", "user-agent"];

/** The headers of an answer that must not be copied, since this server sets them itself. */
const HOP = new Set(["connection", "keep-alive", "transfer-encoding", "content-encoding", "content-length"]);

/** Passes a request on to the API. */
async function forward(request: NextRequest, { params }: { params: Promise<{ path: string[] }> }): Promise<Response> {
  const base = process.env["KAMPUS_API_URL"] ?? "http://localhost:8100";
  const { path } = await params;
  const target = `${base}/api/${path.map(encodeURIComponent).join("/")}${request.nextUrl.search}`;
  const headers = new Headers();
  for (const name of FORWARDED) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  const forwardedFor = request.headers.get("x-forwarded-for");
  if (forwardedFor) headers.set("x-forwarded-for", forwardedFor);
  const body = request.method === "GET" || request.method === "HEAD" ? undefined : await request.arrayBuffer();
  const answer = await fetch(target, { method: request.method, headers, body, redirect: "manual", cache: "no-store" });
  const out = new Headers();
  answer.headers.forEach((value, name) => {
    if (!HOP.has(name) && name !== "set-cookie") out.set(name, value);
  });
  for (const cookie of answer.headers.getSetCookie()) out.append("set-cookie", cookie);
  return new Response(answer.body, { status: answer.status, statusText: answer.statusText, headers: out });
}

export const GET = forward;
export const POST = forward;
export const PUT = forward;
export const PATCH = forward;
export const DELETE = forward;
export const HEAD = forward;
