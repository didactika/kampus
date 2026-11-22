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


/** One failing field of a validation problem: the field and the rule it breaks. */
export interface ProblemField {
  readonly field: string;
  readonly rule: string;
}

/** A problem the API answered with: its status, the code of its problem details, its values and failing fields. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    readonly params: Readonly<Record<string, string>> = {},
    readonly fields: readonly ProblemField[] = [],
  ) {
    super(code);
    this.name = "ApiError";
  }

  /** The rules a field breaks. */
  rulesOf(field: string): string[] {
    return this.fields.filter((f) => f.field === field).map((f) => f.rule);
  }
}

/** Reads an answer of the API: its JSON, or an `ApiError` with the code, values and fields of the problem. */
export async function answer<T>(response: Response): Promise<T> {
  if (response.status === 204) return undefined as T;
  const text = await response.text();
  const data: unknown = text ? JSON.parse(text) : undefined;
  if (!response.ok) {
    const problem = (typeof data === "object" && data !== null ? data : {}) as { code?: unknown; params?: unknown; fields?: unknown };
    const params = typeof problem.params === "object" && problem.params !== null ? (problem.params as Record<string, string>) : {};
    const fields = Array.isArray(problem.fields) ? (problem.fields as ProblemField[]) : [];
    throw new ApiError(response.status, problem.code === undefined ? "unknown" : String(problem.code), params, fields);
  }
  return data as T;
}
