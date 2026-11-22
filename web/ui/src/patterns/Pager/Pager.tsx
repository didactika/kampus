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
 * The pages of a list, under it, as Didactika's lists have them: how many items per page on one side (12, 24, 48 or
 * 96, only those smaller than the list, and All when the list has at most 500 items), and on the other the range
 * shown and the previous and next arrows.
 */

import IconButton from "@mui/material/IconButton";
import Box from "@mui/material/Box";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useId } from "react";

import { Icon } from "../../components/Icon";
import { PillSelect } from "../PillSelect";

/** The sizes a page can have; 0 is the whole list. */
export const PAGE_SIZES = [12, 24, 48, 96] as const;

/** The largest list "All" is offered for. */
const ALL_UP_TO = 500;

/** The sizes offered for a list of this many items. */
export function pageSizes(total: number): number[] {
  const smaller = PAGE_SIZES.filter((size) => size < total);
  return total <= ALL_UP_TO ? [...(smaller.length ? smaller : [PAGE_SIZES[0]]), 0] : smaller;
}

/** The size a page takes: the one asked for when it is offered, the whole list or 12 otherwise. */
export function pageSize(asked: number, total: number): number {
  const offered = pageSizes(total);
  if (offered.includes(asked)) return asked;
  return offered.includes(0) && asked > 0 ? 0 : PAGE_SIZES[0];
}

/** The props of `Pager`. */
export interface PagerProps {
  /** How many items the list has. */
  readonly total: number;
  /** The page shown, from 0. */
  readonly page: number;
  /** The items per page; 0 is the whole list. */
  readonly size: number;
  /** Changes the page. */
  readonly onPage: (page: number) => void;
  /** Changes the size. */
  readonly onSize: (size: number) => void;
  /** The words of the pager. */
  readonly labels: {
    readonly perPage: string;
    readonly all: string;
    readonly previous: string;
    readonly next: string;
    /** The range, as "1–12 of 62". */
    readonly range: (from: number, to: number, total: number) => string;
  };
}

/** `<Pager total={62} page={0} size={12} … />` */
export function Pager({ total, page, size, onPage, onSize, labels }: PagerProps) {
  const select = useId();
  const per = size === 0 ? total : size;
  const pages = Math.max(1, Math.ceil(total / Math.max(per, 1)));
  const from = total === 0 ? 0 : page * per + 1;
  const to = Math.min(total, (page + 1) * per);
  return (
    <Stack direction={{ xs: "column", sm: "row" }} spacing={2} sx={{ alignItems: { sm: "center" }, justifyContent: "space-between", mt: 3, pt: 2, borderTop: 1, borderColor: "divider" }}>
      <Stack direction="row" spacing={1.5} sx={{ alignItems: "center" }}>
        <Typography variant="body2" color="text.secondary" aria-hidden>
          {labels.perPage}
        </Typography>
        <Box id={select}>
          <PillSelect
            label={labels.perPage}
            value={String(size)}
            onChange={(value) => onSize(Number(value))}
            options={pageSizes(total).map((option) => ({ value: String(option), label: option === 0 ? labels.all : String(option) }))}
          />
        </Box>
      </Stack>
      <Stack direction="row" spacing={0.5} sx={{ alignItems: "center" }}>
        <Typography variant="body2" color="text.secondary" aria-live="polite" sx={{ mr: 1 }}>
          {labels.range(from, to, total)}
        </Typography>
        <IconButton aria-label={labels.previous} disabled={page === 0} onClick={() => onPage(page - 1)} sx={{ "&:hover, &:focus-visible": { color: "primary.main" } }}>
          <Icon name="chevron_left" />
        </IconButton>
        <IconButton aria-label={labels.next} disabled={page >= pages - 1} onClick={() => onPage(page + 1)} sx={{ "&:hover, &:focus-visible": { color: "primary.main" } }}>
          <Icon name="chevron_right" />
        </IconButton>
      </Stack>
    </Stack>
  );
}
