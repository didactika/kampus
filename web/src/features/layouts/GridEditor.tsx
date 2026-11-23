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
 * The editor of a page of widgets: each widget is a card dragged by its whole frame to any
 * cell and resized from its edges within the sizes it declares, and the widgets in the way make room below while it
 * moves, so the person sees where everything lands before letting go. The keyboard does the same: the arrow keys move
 * the widget that has the focus by one cell, and shift with the arrow keys resizes it. Every change is reported once
 * it is made; nothing is saved here.
 */

import Box from "@mui/material/Box";
import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent as ReactPointerEvent, type ReactNode } from "react";

import { useI18n } from "../../i18n/I18nProvider";
import { place, reading, step as past, within, type Item, type Sizes } from "./grid/engine";
import { GAP } from "./GridView";
import { WidgetFrame } from "./WidgetFrame";

/** The pixels of one unit of spacing of the theme. */
const UNIT = 8;

/** How far the pointer goes before a press becomes a move, in pixels. */
const THRESHOLD = 5;

/** The rows left free under the last widget, to move one below the others. */
const SPARE_ROWS = 3;

/** A widget being arranged. */
export interface EditedWidget extends Item {
  /** The sizes it declares for the page; none when the page no longer offers it, and then it keeps its size. */
  readonly sizes?: Sizes;
  readonly name: string;
  readonly icon: string;
  /** What it shows, or what stands for it until the page is rendered again. */
  readonly node: ReactNode;
  /** Opens its settings; none when it has none. */
  readonly onSettings?: () => void;
  /** Whether the person may remove it; a widget every page holds is not removed. */
  readonly removable: boolean;
}

/** The props of `GridEditor`. */
export interface GridEditorProps {
  readonly columns: number;
  /** The height of a row, in pixels. */
  readonly row: number;
  readonly items: readonly EditedWidget[];
  /** Reports the rectangles once a move or a resize is done. */
  readonly onChange: (items: readonly Item[]) => void;
  /** Removes a widget. */
  readonly onRemove: (key: number) => void;
  /** Pins a widget where it is, or lets it rise again. */
  readonly onPin: (key: number) => void;
}

/** What the pointer is doing to a widget. */
interface Gesture {
  readonly key: number;
  readonly kind: "move" | "resize";
  /** The edges a resize moves: the right one, the bottom one or both. */
  readonly edges: { readonly right: boolean; readonly bottom: boolean };
  readonly startX: number;
  readonly startY: number;
  /** The rectangles when it began, which every step places from. */
  readonly origin: readonly Item[];
  readonly started: boolean;
  readonly dx: number;
  readonly dy: number;
}

/** `<GridEditor columns={12} row={72} items={…} onChange={…} onRemove={…} />` */
export function GridEditor({ columns, row, items, onChange, onRemove, onPin }: GridEditorProps) {
  const { t } = useI18n();
  const canvas = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  const [gesture, setGesture] = useState<Gesture | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [announced, setAnnounced] = useState("");
  const gap = GAP * UNIT;

  useEffect(() => {
    const element = canvas.current;
    if (!element) return undefined;
    const observer = new ResizeObserver(([entry]) => setWidth(entry?.contentRect.width ?? 0));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const column = width > 0 ? (width - gap * (columns - 1)) / columns : 0;
  const pitchX = column + gap;
  const pitchY = row + gap;
  const box = (item: Pick<Item, "x" | "y" | "w" | "h">) => ({
    left: item.x * pitchX,
    top: item.y * pitchY,
    width: item.w * column + (item.w - 1) * gap,
    height: item.h * row + (item.h - 1) * gap,
  });
  const sizesOf = (key: number) => items.find((i) => i.key === key)?.sizes;

  /** The rectangle the gesture gives the widget, and where every widget then is. */
  function preview(g: Gesture): { readonly target: Item; readonly placed: readonly Item[] } {
    const start = g.origin.find((i) => i.key === g.key)!;
    let to: Item = start;
    if (g.kind === "move") {
      to = { ...start, x: Math.round((start.x * pitchX + g.dx) / pitchX), y: Math.max(0, Math.round((start.y * pitchY + g.dy) / pitchY)) };
    } else {
      const size = {
        w: g.edges.right ? Math.round((box(start).width + g.dx + gap) / pitchX) : start.w,
        h: g.edges.bottom ? Math.round((box(start).height + g.dy + gap) / pitchY) : start.h,
      };
      const sizes = sizesOf(g.key);
      to = { ...start, ...(sizes ? within(sizes, size, columns) : start) };
    }
    const placed = place(g.origin, g.key, to, columns);
    return { target: placed.find((i) => i.key === g.key)!, placed };
  }

  const rectangles = (list: readonly Item[]) => list.map(({ key, x, y, w, h, pinned }) => ({ key, x, y, w, h, pinned }));
  const shown = gesture?.started ? preview(gesture).placed : rectangles(items);
  const at = (key: number) => shown.find((i) => i.key === key)!;
  const bottom = shown.reduce((low, i) => Math.max(low, i.y + i.h), 0) + SPARE_ROWS;

  const say = (item: Item) => {
    const widget = items.find((i) => i.key === item.key);
    setAnnounced(t("grid.position", { widget: widget?.name ?? "", x: item.x + 1, y: item.y + 1, w: item.w, h: item.h }));
  };

  function begin(event: ReactPointerEvent<HTMLElement>, key: number, kind: Gesture["kind"], edges = { right: true, bottom: true }) {
    if (event.button !== 0) return;
    event.stopPropagation();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    setSelected(key);
    setGesture({ key, kind, edges, startX: event.clientX, startY: event.clientY, origin: rectangles(items), started: false, dx: 0, dy: 0 });
  }

  function follow(event: ReactPointerEvent<HTMLElement>) {
    setGesture((g) => {
      if (!g) return g;
      const dx = event.clientX - g.startX;
      const dy = event.clientY - g.startY;
      const started = g.started || Math.hypot(dx, dy) > THRESHOLD;
      return { ...g, dx, dy, started };
    });
  }

  function end() {
    if (gesture?.started) {
      const { placed, target } = preview(gesture);
      onChange(placed);
      say(target);
    }
    setGesture(null);
  }

  /** Arrows move the widget: sideways by a cell, up or down past the next widget; with shift they resize it. */
  function key(event: KeyboardEvent<HTMLElement>, item: EditedWidget) {
    const step = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] }[event.key];
    if (!step || event.target !== event.currentTarget) return;
    event.preventDefault();
    const [dx, dy] = step as [number, number];
    const current = rectangles(items);
    const start = current.find((i) => i.key === item.key)!;
    let to: Item;
    if (event.shiftKey) {
      if (!item.sizes) return;
      to = { ...start, ...within(item.sizes, { w: start.w + dx, h: start.h + dy }, columns) };
    } else if (dy !== 0) {
      const placed = past(current, item.key, dy > 0, columns);
      onChange(placed);
      say(placed.find((i) => i.key === item.key)!);
      return;
    } else {
      to = { ...start, x: start.x + dx };
    }
    const placed = place(current, item.key, to, columns);
    onChange(placed);
    say(placed.find((i) => i.key === item.key)!);
  }

  const lifted = gesture?.started ? gesture : null;
  const resizable = (sizes: Sizes | undefined) => sizes && sizes.resize !== "none";

  return (
    <Box>
      <Box component="p" id="grid-instructions" sx={{ position: "absolute", inlineSize: 1, blockSize: 1, overflow: "hidden", clipPath: "inset(50%)" }}>
        {t("grid.instructions")}
      </Box>
      <Box aria-live="polite" sx={{ position: "absolute", inlineSize: 1, blockSize: 1, overflow: "hidden", clipPath: "inset(50%)" }}>
        {announced}
      </Box>
      <Box
        ref={canvas}
        role="list"
        aria-label={t("grid.label")}
        aria-describedby="grid-instructions"
        sx={{
          position: "relative",
          blockSize: bottom * pitchY - gap,
          borderRadius: 4,
          transition: "block-size 200ms cubic-bezier(0.2, 0, 0, 1)",
        }}
      >
        {width > 0 && lifted
          ? // While a widget moves or grows, the cells of the grid show, so the person sees where it can go.
            Array.from({ length: bottom * columns }, (_, cell) => (
              <Box
                key={cell}
                aria-hidden
                sx={{
                  position: "absolute",
                  ...box({ x: cell % columns, y: Math.floor(cell / columns), w: 1, h: 1 }),
                  borderRadius: 2,
                  bgcolor: "color-mix(in srgb, var(--k-palette-primary-main) 6%, transparent)",
                  animation: "kampus-cells-in 160ms ease-out",
                  "@keyframes kampus-cells-in": { from: { opacity: 0 }, to: { opacity: 1 } },
                }}
              />
            ))
          : null}
        {width > 0 && lifted ? (
          <Box
            aria-hidden
            sx={{
              position: "absolute",
              ...box(preview(lifted).target),
              borderRadius: 4,
              border: "2px dashed",
              borderColor: "primary.main",
              bgcolor: "color-mix(in srgb, var(--k-palette-primary-main) 8%, transparent)",
              transition: "left 120ms, top 120ms, width 120ms, height 120ms",
            }}
          />
        ) : null}
        {width > 0
          ? reading(items).map((item) => {
              const moving = lifted?.key === item.key;
              const rect = box(at(item.key));
              const start = moving ? box(lifted.origin.find((i) => i.key === item.key)!) : rect;
              const style = moving
                ? lifted.kind === "move"
                  ? { left: start.left + lifted.dx, top: start.top + lifted.dy, width: start.width, height: start.height, zIndex: 3 }
                  : { ...rect, zIndex: 3 }
                : { ...rect, zIndex: selected === item.key ? 2 : 1 };
              return (
                <Box
                  key={item.key}
                  role="listitem"
                  sx={{
                    position: "absolute",
                    ...style,
                    // The corner that resizes shows only while the pointer or the focus is on the widget.
                    "&:hover .resize-corner::after, &:focus-within .resize-corner::after": { opacity: 0.55 },
                    transition: moving ? "none" : "left 200ms cubic-bezier(0.2, 0, 0, 1), top 200ms cubic-bezier(0.2, 0, 0, 1), width 200ms cubic-bezier(0.2, 0, 0, 1), height 200ms cubic-bezier(0.2, 0, 0, 1)",
                    "@media (prefers-reduced-motion: reduce)": { transition: "none" },
                  }}
                >
                  <WidgetFrame
                    name={item.name}
                    icon={item.icon}
                    fixed={!resizable(item.sizes)}
                    selected={selected === item.key}
                    lifted={moving && lifted.kind === "move"}
                    tabIndex={0}
                    role="group"
                    aria-roledescription={t("grid.widget")}
                    aria-label={t("grid.position", { widget: item.name, x: item.x + 1, y: item.y + 1, w: item.w, h: item.h })}
                    onFocus={() => setSelected(item.key)}
                    onPointerDown={(event) => begin(event, item.key, "move")}
                    onPointerMove={follow}
                    onPointerUp={end}
                    onPointerCancel={() => setGesture(null)}
                    onKeyDown={(event) => key(event, item)}
                    onSettings={item.onSettings}
                    onRemove={item.removable ? () => onRemove(item.key) : undefined}
                    pinned={Boolean(item.pinned)}
                    onPin={() => onPin(item.key)}
                  >
                    {item.node}
                  </WidgetFrame>
                  {resizable(item.sizes) ? (
                    <ResizeHandles
                      name={t("grid.resize", { widget: item.name })}
                      resize={item.sizes!.resize}
                      onPointerDown={(event, edges) => begin(event, item.key, "resize", edges)}
                      onPointerMove={follow}
                      onPointerUp={end}
                      onPointerCancel={() => setGesture(null)}
                    />
                  ) : null}
                </Box>
              );
            })
          : null}
      </Box>
    </Box>
  );
}

/** The props of `ResizeHandles`. */
interface ResizeHandlesProps {
  readonly name: string;
  readonly resize: Sizes["resize"];
  readonly onPointerDown: (event: ReactPointerEvent<HTMLElement>, edges: Gesture["edges"]) => void;
  readonly onPointerMove: (event: ReactPointerEvent<HTMLElement>) => void;
  readonly onPointerUp: () => void;
  readonly onPointerCancel: () => void;
}

/** The edges of a widget that resize it: the right one, the bottom one and the corner between, as it allows. */
function ResizeHandles({ name, resize, onPointerDown, onPointerMove, onPointerUp, onPointerCancel }: ResizeHandlesProps) {
  const horizontal = resize === "both" || resize === "horizontal";
  const vertical = resize === "both" || resize === "vertical";
  const common = { onPointerMove, onPointerUp, onPointerCancel, "aria-hidden": true as const, title: name };
  const edge = { position: "absolute", zIndex: 4, touchAction: "none", "&:hover::after, &:active::after": { opacity: 1 } } as const;
  const bar = { content: '""', position: "absolute", borderRadius: 1, bgcolor: "primary.main", opacity: 0, transition: "opacity 120ms" } as const;
  return (
    <>
      {horizontal ? (
        <Box
          {...common}
          onPointerDown={(event) => onPointerDown(event, { right: true, bottom: false })}
          sx={{ ...edge, insetBlock: 16, insetInlineEnd: -6, inlineSize: 12, cursor: "ew-resize", "&::after": { ...bar, insetBlock: "30%", insetInlineStart: 4, inlineSize: 4 } }}
        />
      ) : null}
      {vertical ? (
        <Box
          {...common}
          onPointerDown={(event) => onPointerDown(event, { right: false, bottom: true })}
          sx={{ ...edge, insetInline: 16, insetBlockEnd: -6, blockSize: 12, cursor: "ns-resize", "&::after": { ...bar, insetInline: "35%", insetBlockStart: 4, blockSize: 4 } }}
        />
      ) : null}
      {horizontal && vertical ? (
        <Box
          {...common}
          className="resize-corner"
          onPointerDown={(event) => onPointerDown(event, { right: true, bottom: true })}
          sx={{
            ...edge,
            insetInlineEnd: -4,
            insetBlockEnd: -4,
            inlineSize: 22,
            blockSize: 22,
            cursor: "nwse-resize",
            "&::after": { ...bar, insetInlineEnd: 6, insetBlockEnd: 6, inlineSize: 10, blockSize: 10, bgcolor: "transparent", borderInlineEnd: "3px solid", borderBlockEnd: "3px solid", borderColor: "primary.main", borderEndEndRadius: 6 },
          }}
        />
      ) : null}
    </>
  );
}
