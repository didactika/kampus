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
import { rows, spots } from "./grid/spots";
import type { EditInPlaceValue } from "../editor/EditInPlace";
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
  /** Its settings and how to change one, for the texts it edits in place. */
  readonly edit?: EditInPlaceValue;
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

/** The edges of a widget a resize moves. */
interface Edges {
  readonly left?: boolean;
  readonly right?: boolean;
  readonly top?: boolean;
  readonly bottom?: boolean;
}

/** What the pointer is doing to a widget. */
interface Gesture {
  readonly key: number;
  readonly kind: "move" | "resize";
  /** The edges a resize moves: one side, or the two of a corner. */
  readonly edges: Edges;
  readonly startX: number;
  readonly startY: number;
  /** The rectangles when it began, which every step places from. */
  readonly origin: readonly Item[];
  readonly started: boolean;
  readonly dx: number;
  readonly dy: number;
}

/**
 * The height the content of a widget needs, in pixels: what it shows, plus what the lists inside it hide below their
 * fold.
 */
function natural(element: HTMLElement): number {
  let hidden = 0;
  for (const inner of element.querySelectorAll<HTMLElement>("*")) {
    if (inner.scrollHeight > inner.clientHeight + 1 && /(auto|scroll)/.test(getComputedStyle(inner).overflowY)) {
      hidden = Math.max(hidden, inner.scrollHeight - inner.clientHeight);
    }
  }
  return Math.max(element.scrollHeight, element.offsetHeight) + hidden;
}

/** `<GridEditor columns={12} row={72} items={…} onChange={…} onRemove={…} />` */
export function GridEditor({ columns, row, items, onChange, onRemove, onPin }: GridEditorProps) {
  const { t } = useI18n();
  const canvas = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  const [gesture, setGesture] = useState<Gesture | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [announced, setAnnounced] = useState("");
  const contents = useRef(new Map<number, HTMLDivElement>());
  const [heights, setHeights] = useState<Readonly<Record<number, number>>>({});
  const [naturals, setNaturals] = useState<Readonly<Record<number, number>>>({});
  const gap = GAP * UNIT;

  useEffect(() => {
    const element = canvas.current;
    if (!element) return undefined;
    const observer = new ResizeObserver(([entry]) => setWidth(entry?.contentRect.width ?? 0));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  // The height each widget shows, measured as the page measures it.
  useEffect(() => {
    const observer = new ResizeObserver((entries) => {
      setHeights((known) => {
        const next = { ...known };
        for (const entry of entries) next[Number((entry.target as HTMLElement).dataset["key"])] = (entry.target as HTMLElement).offsetHeight;
        return next;
      });
      setNaturals((known) => {
        const next = { ...known };
        for (const entry of entries) next[Number((entry.target as HTMLElement).dataset["key"])] = natural(entry.target as HTMLElement);
        return next;
      });
    });
    for (const element of contents.current.values()) observer.observe(element);
    return () => observer.disconnect();
    // The widgets render once the width of the page is known, so they are observed again then.
  }, [items, width > 0]);

  const column = width > 0 ? (width - gap * (columns - 1)) / columns : 0;
  const pitchX = column + gap;
  const pitchY = row + gap;
  const metrics = { columns, row, gap, width };
  const box = (item: Pick<Item, "x" | "y" | "w" | "h">) => ({
    left: item.x * pitchX,
    top: item.y * pitchY,
    width: item.w * column + (item.w - 1) * gap,
    height: rows(metrics, item.h),
  });
  const sizesOf = (key: number) => items.find((i) => i.key === key)?.sizes;
  /** The most rows a widget may take: those its content fills, so it never grows past what it shows. */
  const fill = (key: number, size: { readonly w: number; readonly h: number }, low: number) => {
    const content = naturals[key];
    if (content === undefined) return size;
    return { ...size, h: Math.max(low, Math.min(size.h, Math.ceil((content + gap) / pitchY))) };
  };
  // A widget shows the height of its content, as on the page; the one being resized shows the rows it will take.
  const heightOf = (resizing: number | null) => (item: Item) => (item.key === resizing ? rows(metrics, item.h) : (heights[item.key] ?? rows(metrics, item.h)));
  const layout = (list: readonly Item[], resizing: number | null = null) => spots(list, metrics, heightOf(resizing));

  /** The rectangle the gesture gives the widget, and where every widget then is. */
  function preview(g: Gesture): { readonly target: Item; readonly placed: readonly Item[] } {
    const start = g.origin.find((i) => i.key === g.key)!;
    const from = layout(g.origin).at.get(g.key) ?? box(start);
    let to: Item = start;
    if (g.kind === "move") {
      to = { ...start, x: Math.round((from.left + g.dx) / pitchX), y: Math.max(0, Math.round((from.top + g.dy) / pitchY)) };
    } else {
      // The edge the person holds moves; the opposite one stays where it was.
      const { left, right, top, bottom } = g.edges;
      const height = rows(metrics, start.h);
      const wide = left ? from.width - g.dx : right ? from.width + g.dx : from.width;
      const tall = top ? height - g.dy : bottom ? height + g.dy : height;
      const size = { w: Math.round((wide + gap) / pitchX), h: Math.round((tall + gap) / pitchY) };
      const sizes = sizesOf(g.key);
      const kept = sizes ? fill(g.key, within(sizes, size, columns), Math.min(start.h, sizes.min.h)) : { w: start.w, h: start.h };
      to = {
        ...start,
        ...kept,
        x: left ? Math.max(0, start.x + start.w - kept.w) : start.x,
        y: top ? Math.max(0, start.y + start.h - kept.h) : start.y,
      };
    }
    const placed = place(g.origin, g.key, to, columns);
    return { target: placed.find((i) => i.key === g.key)!, placed };
  }

  const rectangles = (list: readonly Item[]) => list.map(({ key, x, y, w, h, pinned }) => ({ key, x, y, w, h, pinned }));
  const lifted = gesture?.started ? gesture : null;
  const shown = lifted ? preview(lifted).placed : rectangles(items);
  const now = layout(shown, lifted?.kind === "resize" ? lifted.key : null);
  const origin = lifted ? layout(lifted.origin) : now;

  const say = (item: Item) => {
    const widget = items.find((i) => i.key === item.key);
    setAnnounced(t("grid.position", { widget: widget?.name ?? "", x: item.x + 1, y: item.y + 1, w: item.w, h: item.h }));
  };

  function begin(event: ReactPointerEvent<HTMLElement>, key: number, kind: Gesture["kind"], edges: Edges = {}) {
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
      to = { ...start, ...fill(item.key, within(item.sizes, { w: start.w + dx, h: start.h + dy }, columns), Math.min(start.h, item.sizes.min.h)) };
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

  const resizable = (sizes: Sizes | undefined) => sizes && sizes.resize !== "none";
  // While a widget moves, the page keeps a few rows below the last widget to drop it there.
  const bottom = now.bottom + (lifted ? SPARE_ROWS * pitchY : 0);

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
        sx={{ position: "relative", blockSize: bottom, transition: "block-size 200ms cubic-bezier(0.2, 0, 0, 1)" }}
      >
        {width > 0 && lifted
          ? // While a widget moves or grows, the cells of the grid show, so the person sees where it can go.
            Array.from({ length: Math.ceil(bottom / pitchY) * columns }, (_, cell) => (
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
              ...now.at.get(lifted.key),
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
              const spot = now.at.get(item.key)!;
              const start = origin.at.get(item.key) ?? spot;
              const style =
                moving && lifted.kind === "move"
                  ? { left: start.left + lifted.dx, top: start.top + lifted.dy, width: start.width, height: start.height, zIndex: 3 }
                  : { ...spot, zIndex: moving ? 3 : selected === item.key ? 2 : 1 };
              return (
                <Box
                  key={item.key}
                  role="listitem"
                  sx={{
                    position: "absolute",
                    ...style,
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
                    contentRef={(element: HTMLDivElement | null) => {
                      if (element) {
                        element.dataset["key"] = String(item.key);
                        contents.current.set(item.key, element);
                      } else contents.current.delete(item.key);
                    }}
                    maxContent={rows(metrics, item.h)}
                    onFocus={() => setSelected(item.key)}
                    onPointerDown={(event) => begin(event, item.key, "move")}
                    onPointerMove={follow}
                    onPointerUp={end}
                    onPointerCancel={() => setGesture(null)}
                    onKeyDown={(event) => key(event, item)}
                    onSettings={item.onSettings}
                    edit={item.edit}
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
  readonly onPointerDown: (event: ReactPointerEvent<HTMLElement>, edges: Edges) => void;
  readonly onPointerMove: (event: ReactPointerEvent<HTMLElement>) => void;
  readonly onPointerUp: () => void;
  readonly onPointerCancel: () => void;
}

/** The thickness of the band along an edge that resizes, and the side of a corner, in pixels. */
const EDGE = 10;
const CORNER = 22;

/** The radius of a card, which a highlighted corner follows. */
const RADIUS = 16;

/** A handle: the edges it moves, where it sits and the cursor it shows. */
interface Handle {
  readonly edges: Edges;
  readonly cursor: string;
  readonly at: Record<string, number | string>;
  /** The part of the border it lights up. */
  readonly border: Record<string, number | string>;
}

/** Every handle a widget may have: four sides and four corners. */
const HANDLES: readonly Handle[] = [
  { edges: { right: true }, cursor: "ew-resize", at: { insetBlock: CORNER, insetInlineEnd: -EDGE / 2, inlineSize: EDGE }, border: { insetBlock: 0, insetInlineEnd: EDGE / 2 - 1, inlineSize: 3 } },
  { edges: { left: true }, cursor: "ew-resize", at: { insetBlock: CORNER, insetInlineStart: -EDGE / 2, inlineSize: EDGE }, border: { insetBlock: 0, insetInlineStart: EDGE / 2 - 1, inlineSize: 3 } },
  { edges: { bottom: true }, cursor: "ns-resize", at: { insetInline: CORNER, insetBlockEnd: -EDGE / 2, blockSize: EDGE }, border: { insetInline: 0, insetBlockEnd: EDGE / 2 - 1, blockSize: 3 } },
  { edges: { top: true }, cursor: "ns-resize", at: { insetInline: CORNER, insetBlockStart: -EDGE / 2, blockSize: EDGE }, border: { insetInline: 0, insetBlockStart: EDGE / 2 - 1, blockSize: 3 } },
  { edges: { right: true, bottom: true }, cursor: "nwse-resize", at: { insetInlineEnd: -EDGE / 2, insetBlockEnd: -EDGE / 2, inlineSize: CORNER, blockSize: CORNER }, border: { insetInlineEnd: EDGE / 2 - 1, insetBlockEnd: EDGE / 2 - 1, inlineSize: CORNER, blockSize: CORNER, borderInlineEnd: "3px solid", borderBlockEnd: "3px solid", borderEndEndRadius: RADIUS } },
  { edges: { left: true, bottom: true }, cursor: "nesw-resize", at: { insetInlineStart: -EDGE / 2, insetBlockEnd: -EDGE / 2, inlineSize: CORNER, blockSize: CORNER }, border: { insetInlineStart: EDGE / 2 - 1, insetBlockEnd: EDGE / 2 - 1, inlineSize: CORNER, blockSize: CORNER, borderInlineStart: "3px solid", borderBlockEnd: "3px solid", borderEndStartRadius: RADIUS } },
  { edges: { right: true, top: true }, cursor: "nesw-resize", at: { insetInlineEnd: -EDGE / 2, insetBlockStart: -EDGE / 2, inlineSize: CORNER, blockSize: CORNER }, border: { insetInlineEnd: EDGE / 2 - 1, insetBlockStart: EDGE / 2 - 1, inlineSize: CORNER, blockSize: CORNER, borderInlineEnd: "3px solid", borderBlockStart: "3px solid", borderStartEndRadius: RADIUS } },
  { edges: { left: true, top: true }, cursor: "nwse-resize", at: { insetInlineStart: -EDGE / 2, insetBlockStart: -EDGE / 2, inlineSize: CORNER, blockSize: CORNER }, border: { insetInlineStart: EDGE / 2 - 1, insetBlockStart: EDGE / 2 - 1, inlineSize: CORNER, blockSize: CORNER, borderInlineStart: "3px solid", borderBlockStart: "3px solid", borderStartStartRadius: RADIUS } },
];

/**
 * The edges and corners of a widget that resize it, as far as it may be resized. They are invisible: only the one
 * under the pointer lights up its part of the border of the card, so the person sees what they will pull.
 */
function ResizeHandles({ name, resize, onPointerDown, onPointerMove, onPointerUp, onPointerCancel }: ResizeHandlesProps) {
  const horizontal = resize === "both" || resize === "horizontal";
  const vertical = resize === "both" || resize === "vertical";
  const allowed = (e: Edges) => (horizontal || !(e.left || e.right)) && (vertical || !(e.top || e.bottom));
  return (
    <>
      {HANDLES.filter((h) => allowed(h.edges)).map((handle) => (
        <Box
          key={handle.cursor + Object.keys(handle.edges).join()}
          aria-hidden
          title={name}
          onPointerDown={(event) => onPointerDown(event, handle.edges)}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerCancel}
          sx={{
            position: "absolute",
            zIndex: 4,
            touchAction: "none",
            cursor: handle.cursor,
            ...handle.at,
            "&::after": {
              content: '""',
              position: "absolute",
              ...handle.border,
              ...(handle.border["borderInlineStart"] || handle.border["borderInlineEnd"] || handle.border["borderBlockStart"] || handle.border["borderBlockEnd"]
                ? { borderColor: "primary.main", bgcolor: "transparent" }
                : { bgcolor: "primary.main", borderRadius: 2 }),
              opacity: 0,
              transition: "opacity 120ms",
            },
            "&:hover::after, &:active::after": { opacity: 1 },
          }}
        />
      ))}
    </>
  );
}
