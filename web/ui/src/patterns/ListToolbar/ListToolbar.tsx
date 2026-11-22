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
 * The toolbar of a list, as Didactika's lists have it: a search that applies while the person types (after a short
 * pause) and keeps the cursor where it is; right after it, the Filters button with the number of filters on, which
 * opens the filters with a title and a reset; then the controls of the view; and the actions of the page at the end.
 */

import Badge from "@mui/material/Badge";
import Box from "@mui/material/Box";
import IconButton from "@mui/material/IconButton";
import InputAdornment from "@mui/material/InputAdornment";
import OutlinedInput from "@mui/material/OutlinedInput";
import Popover from "@mui/material/Popover";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { useEffect, useId, useRef, useState, type ReactNode } from "react";

import { Button } from "../../components/Button";
import { Icon } from "../../components/Icon";

/** The props of `ListToolbar`. */
export interface ListToolbarProps {
  /** The text searched. */
  readonly search: string;
  /** Applies a search, after the person stops typing. */
  readonly onSearch: (text: string) => void;
  /** The name and hint of the search field. */
  readonly searchLabel: string;
  /** The filters, shown in the drop-down of the Filters button. */
  readonly filters?: ReactNode;
  /** How many filters are on. */
  readonly activeFilters?: number;
  /** The words of the Filters button, of the head of its drop-down and of its reset. */
  readonly filtersLabel?: string;
  readonly resetLabel?: string;
  /** Clears every filter. */
  readonly onReset?: () => void;
  /** Controls of the view, such as the order and the layout. */
  readonly view?: ReactNode;
  /** The actions of the page. */
  readonly actions?: ReactNode;
}

/** How long the search waits after the last key, in milliseconds. */
const PAUSE = 400;

/** `<ListToolbar search={q} onSearch={setQ} searchLabel="Search courses" filters={…} />` */
export function ListToolbar(props: ListToolbarProps) {
  const { search, onSearch, searchLabel, filters, activeFilters = 0, filtersLabel = "Filters", resetLabel = "Reset", onReset, view, actions } = props;
  const [text, setText] = useState(search);
  const [anchor, setAnchor] = useState<HTMLElement | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const panel = useId();

  useEffect(() => setText(search), [search]);
  useEffect(() => () => clearTimeout(timer.current), []);

  function typed(value: string) {
    setText(value);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => onSearch(value.trim()), PAUSE);
  }

  return (
    <Stack direction={{ xs: "column", md: "row" }} spacing={1} sx={{ alignItems: { md: "center" }, mb: 3 }}>
      <Stack direction="row" spacing={1} sx={{ flex: 1, alignItems: "center", minInlineSize: 0 }}>
        <OutlinedInput
          value={text}
          onChange={(event) => typed(event.target.value)}
          placeholder={searchLabel}
          inputProps={{ "aria-label": searchLabel, type: "search" }}
          startAdornment={
            <InputAdornment position="start">
              <Icon name="search" size="small" />
            </InputAdornment>
          }
          sx={{ flex: "0 1 340px", minInlineSize: 180, borderRadius: (theme) => `${theme.shape.full}px`, pl: 1.5, "& input": { pl: 0.5 } }}
        />
        {filters ? (
          <>
            <Badge badgeContent={activeFilters} color="primary" overlap="circular">
              <Button
                variant={activeFilters > 0 ? "tonal" : "outlined"}
                icon="tune"
                aria-haspopup="dialog"
                aria-expanded={anchor ? true : undefined}
                aria-controls={anchor ? panel : undefined}
                onClick={(event) => setAnchor(event.currentTarget)}
                sx={{ borderRadius: (theme) => `${theme.shape.full}px`, px: 2 }}
              >
                {filtersLabel}
              </Button>
            </Badge>
            <Popover
              id={panel}
              open={Boolean(anchor)}
              anchorEl={anchor}
              onClose={() => setAnchor(null)}
              anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
              slotProps={{ paper: { role: "dialog", "aria-label": filtersLabel, sx: { p: 2.5, mt: 1, inlineSize: 360, maxInlineSize: "calc(100vw - 32px)", borderRadius: 3 } } }}
            >
              <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", mb: 1.5 }}>
                <Typography variant="subtitle1">{filtersLabel}</Typography>
                {onReset ? (
                  <Tooltip title={resetLabel}>
                    <span>
                      <IconButton aria-label={resetLabel} onClick={onReset} disabled={activeFilters === 0}>
                        <Icon name="filter_alt_off" size="small" />
                      </IconButton>
                    </span>
                  </Tooltip>
                ) : null}
              </Box>
              <Stack spacing={2.5}>{filters}</Stack>
            </Popover>
          </>
        ) : null}
      </Stack>
      {view || actions ? (
        <Stack direction="row" spacing={1} sx={{ alignItems: "center", justifyContent: "flex-end", flexWrap: "wrap", rowGap: 1 }}>
          {view}
          {actions}
        </Stack>
      ) : null}
    </Stack>
  );
}
