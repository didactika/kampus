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
 * A page of widgets on a grid, such as the home of a campus or the page of a course. For a person who may arrange it,
 * the edit mode of the bar turns it into its editor: widgets are dragged to any cell and resized within the sizes they
 * declare, their settings open at the side, new ones go in the first room that holds them, and the arrangement can go
 * back to the default. Every change is saved as it is made.
 */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardActionArea from "@mui/material/CardActionArea";
import Dialog from "@mui/material/Dialog";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Grid from "@mui/material/Grid";
import Skeleton from "@mui/material/Skeleton";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useTheme } from "@mui/material/styles";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";

import { Button, ConfirmDialog, Icon, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { AvailableWidget, DeclaredSetting, Placement } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { useOfferEditMode } from "../shell/EditMode";
import { useLeaveGuard } from "../shell/LeaveGuard";
import { room, settle, type Item } from "./grid/engine";
import { GridEditor } from "./GridEditor";
import { GridView } from "./GridView";
import { WidgetFrame } from "./WidgetFrame";
import { WidgetSettings } from "./WidgetSettings";

/** A widget placed on the page, with what it shows when it was rendered by the server. */
export interface PlacedWidget {
  readonly placement: Placement;
  readonly node: ReactNode;
}

/** A widget while the page is arranged; one added or changed since has nothing to show until it is rendered again. */
interface Draft {
  readonly key: number;
  readonly placement: Placement;
  readonly node: ReactNode | null;
}

/** The grid of a page: its columns and the height of a row, in pixels. */
export interface PageGrid {
  readonly columns: number;
  readonly row: number;
}

/** The props of `ArrangedPage`. */
export interface ArrangedPageProps {
  /** The scope the page is arranged for. */
  readonly scope: string;
  /** The page kind. */
  readonly pageKind: string;
  /** Its grid. */
  readonly grid: PageGrid;
  /** Its widgets, as rendered. */
  readonly widgets: readonly PlacedWidget[];
  /** Whether the reader may arrange it. */
  readonly editable: boolean;
  /** The widgets the reader may add, with their sizes on the page. */
  readonly available: readonly AvailableWidget[];
}

let keys = 0;

/** The widgets of a page with every one not pinned risen to the ones above it, as the server keeps them. */
function fall(items: readonly Draft[]): Draft[] {
  const settled = settle(items.map((d) => ({ key: d.key, x: d.placement.x, y: d.placement.y, w: d.placement.w, h: d.placement.h, pinned: d.placement.pinned })));
  return items.map((d) => ({ ...d, placement: { ...d.placement, y: settled.find((s) => s.key === d.key)!.y } }));
}

/** The settings a widget declares, as the API sends them. */
const settingsOf = (widget: AvailableWidget | undefined): DeclaredSetting[] => (widget?.settings ?? []) as DeclaredSetting[];

/** `<ArrangedPage scope="…" pageKind="customise.dashboard" grid={{ columns: 12, row: 72 }} widgets={…} … />` */
export function ArrangedPage({ scope, pageKind, grid, widgets, editable, available }: ArrangedPageProps) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const theme = useTheme();
  const wide = useMediaQuery(theme.breakpoints.up("md"), { noSsr: true });
  const editing = useOfferEditMode(editable);
  const fresh = () => widgets.map((w) => ({ key: ++keys, placement: w.placement, node: w.node }));
  const [draft, setDraft] = useState<Draft[]>(fresh);
  const [preparing, setPreparing] = useState<AvailableWidget | null>(null);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState(false);
  const [adding, setAdding] = useState(false);
  const [configuring, setConfiguring] = useState<number | null>(null);
  const [resetting, setResetting] = useState(false);
  const [changed, setChanged] = useState(0);
  const latest = useRef<Draft[]>([]);
  latest.current = draft;

  const declared = (code: string) => available.find((w) => w.code === code);
  // A widget names and describes itself in the strings of its plugin, and gives its icon in its declaration.
  const name = (code: string) => (t(`${code}.name`) === `${code}.name` ? code : t(`${code}.name`));
  const about = (widget: AvailableWidget) => (t(`${widget.code}.about`) === `${widget.code}.about` ? widget.summary : t(`${widget.code}.about`));
  const iconOf = (code: string) => declared(code)?.icon || "widgets";

  // The arrangement follows the page each time the server renders it again, unless a change is waiting to be saved.
  useEffect(() => {
    if (!pending) setDraft(fresh());
  }, [widgets]);

  /** Saves the arrangement as it is now; content is asked again only when a widget needs it. */
  const save = useCallback(async () => {
    const items = latest.current;
    setPending(false);
    try {
      await browserApi(`/customise/scopes/${scope}/layouts/${pageKind}`, {
        method: "PUT",
        body: {
          placements: items.map(({ placement: { widget, x, y, w, h, pinned, settings } }) => ({ widget, x, y, w, h, pinned, settings })),
        },
      });
      if (items.some((d) => d.node === null)) router.refresh();
    } catch {
      toast(t("dashboard.savefailed"), "danger");
    }
  }, [scope, pageKind, router, toast, t]);

  // Every change is saved a moment after it is made, so several quick ones go together.
  useEffect(() => {
    if (changed === 0) return undefined;
    setPending(true);
    const timer = setTimeout(() => void save(), 400);
    return () => clearTimeout(timer);
  }, [changed, save]);

  // Leaving the mode saves what is still pending at once, then shows the page as it now is, without a reload.
  const wasEditing = useRef(editing);
  useEffect(() => {
    if (wasEditing.current && !editing) void (pending ? save() : Promise.resolve()).then(() => router.refresh());
    wasEditing.current = editing;
  }, [editing, pending, save, router]);

  /** Records a change of the person, to be saved. */
  const change = (update: (items: Draft[]) => Draft[]) => {
    setDraft(update);
    setChanged((n) => n + 1);
  };

  /** Moves and resizes the widgets to the rectangles of the editor; a widget whose size changed is rendered again. */
  const arrange = (rectangles: readonly Item[]) =>
    change((items) =>
      items.map((d) => {
        const r = rectangles.find((x) => x.key === d.key);
        if (!r) return d;
        const resized = r.w !== d.placement.w || r.h !== d.placement.h;
        return { ...d, placement: { ...d.placement, x: r.x, y: r.y, w: r.w, h: r.h, pinned: Boolean(r.pinned) }, node: resized ? null : d.node };
      }),
    );

  /** Puts a widget on the page at its default size, in the first room that holds it. */
  const insert = (widget: AvailableWidget, settings: Record<string, unknown>) =>
    change((items) => {
      const size = { w: Math.min(widget.sizes.default.w, grid.columns), h: widget.sizes.default.h };
      const at = room(items.map((d) => ({ key: d.key, ...d.placement })), size, grid.columns);
      return fall([...items, { key: ++keys, placement: { widget: widget.code, ...at, ...size, pinned: false, settings, permission: null }, node: null }]);
    });

  /** Pins a widget where it is, or lets it rise again with the others. */
  const pin = (key: number) =>
    change((items) => fall(items.map((d) => (d.key === key ? { ...d, placement: { ...d.placement, pinned: !d.placement.pinned } } : d))));

  async function reset() {
    setBusy(true);
    try {
      await browserApi(`/customise/scopes/${scope}/layouts/${pageKind}`, { method: "DELETE" });
      toast(t("dashboard.restored"), "success");
      setResetting(false);
      setPending(false);
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  // A widget a page holds once is not offered again while it is on the page.
  const offered = available.filter((w) => !(w.single && draft.some((d) => d.placement.widget === w.code)));
  const guard = useLeaveGuard(pending);
  const configured = draft.find((d) => d.key === configuring);
  /** The shape of a widget while its content is rendered, never a message. */
  const shape = (
    <Stack spacing={1.25} sx={{ py: 1 }} role="status" aria-label={t("loading")}>
      <Skeleton variant="rounded" height={26} width="45%" animation="wave" />
      <Skeleton variant="rounded" height={64} animation="wave" />
      <Skeleton variant="rounded" height={64} animation="wave" />
    </Stack>
  );
  const rectangle = (d: Draft) => ({ key: d.key, x: d.placement.x, y: d.placement.y, w: d.placement.w, h: d.placement.h, pinned: d.placement.pinned });
  const remove = (key: number) => change((d) => fall(d.filter((x) => x.key !== key)));
  const framed = (d: Draft) => ({
    ...rectangle(d),
    sizes: declared(d.placement.widget)?.sizes,
    name: name(d.placement.widget),
    icon: iconOf(d.placement.widget),
    node: d.node ?? shape,
    onSettings: settingsOf(declared(d.placement.widget)).length > 0 ? () => setConfiguring(d.key) : undefined,
    // A widget every page holds is never removed.
    removable: !declared(d.placement.widget)?.required,
    // Its texts are edited on it: each change is a change of its settings, saved as any other.
    edit: {
      settings: d.placement.settings,
      declared: settingsOf(declared(d.placement.widget)),
      set: (key: string, value: unknown) =>
        change((items) => items.map((x) => (x.key === d.key ? { ...x, placement: { ...x.placement, settings: { ...x.placement.settings, [key]: value } } } : x))),
    },
  });

  return (
    <>
      {editing ? (
        <Stack direction="row" spacing={1} role="toolbar" aria-label={t("editmode.label")} sx={{ justifyContent: "flex-end", alignItems: "center", flexWrap: "wrap", gap: 1, mb: 2 }}>
          <Button variant="filled" icon="add_circle" onClick={() => setAdding(true)} disabled={offered.length === 0}>
            {t("dashboard.add")}
          </Button>
          <Tooltip title={t("dashboard.reset.hint")}>
            <span>
              <Button variant="tonal" icon="restart_alt" onClick={() => setResetting(true)} disabled={busy}>
                {t("dashboard.reset")}
              </Button>
            </span>
          </Tooltip>
        </Stack>
      ) : null}
      {editing && wide ? (
        <GridEditor columns={grid.columns} row={grid.row} items={draft.map(framed)} onChange={arrange} onRemove={remove} onPin={pin} />
      ) : editing ? (
        <>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            {t("grid.narrow")}
          </Typography>
          <GridView
            columns={grid.columns}
            row={grid.row}
            items={draft.map((d) => {
              const f = framed(d);
              return {
                ...rectangle(d),
                node: (
                  <WidgetFrame name={f.name} icon={f.icon} onSettings={f.onSettings} onRemove={f.removable ? () => remove(d.key) : undefined} style={{ cursor: "default", blockSize: 320 }}>
                    {f.node}
                  </WidgetFrame>
                ),
              };
            })}
          />
        </>
      ) : (
        <GridView columns={grid.columns} row={grid.row} items={draft.map((d) => ({ ...rectangle(d), node: d.node }))} />
      )}

      <Dialog open={adding} onClose={() => setAdding(false)} maxWidth="sm" fullWidth aria-labelledby="add-widget">
        <DialogTitle id="add-widget">{t("dashboard.add.title")}</DialogTitle>
        <DialogContent>
          <Grid container spacing={1.5}>
            {offered.map((widget) => (
              <Grid key={widget.code} size={{ xs: 12, sm: 6 }}>
                <Card sx={{ blockSize: "100%" }}>
                  <CardActionArea
                    sx={{ p: 2, blockSize: "100%", display: "flex", alignItems: "flex-start", justifyContent: "flex-start", gap: 1.5 }}
                    onClick={() => {
                      setAdding(false);
                      if (settingsOf(widget).length > 0) setPreparing(widget);
                      else insert(widget, {});
                    }}
                  >
                    <Box sx={{ display: "grid", placeItems: "center", inlineSize: 40, blockSize: 40, flex: "none", borderRadius: 2.5, backgroundImage: "var(--k-palette-gradient-soft)", color: "primary.main" }}>
                      <Icon name={iconOf(widget.code)} />
                    </Box>
                    <Box>
                      <Typography variant="subtitle2">{name(widget.code)}</Typography>
                      <Typography variant="caption" color="text.secondary">
                        {about(widget)}
                      </Typography>
                    </Box>
                  </CardActionArea>
                </Card>
              </Grid>
            ))}
          </Grid>
        </DialogContent>
      </Dialog>

      {preparing ? (
        <WidgetSettings
          title={t("dashboard.add.widget", { widget: name(preparing.code) })}
          icon={iconOf(preparing.code)}
          applyLabel={t("dashboard.insert")}
          declared={settingsOf(preparing)}
          values={Object.fromEntries(settingsOf(preparing).map((s) => [s.key, s.default]))}
          onClose={() => setPreparing(null)}
          onApply={(settings) => {
            insert(preparing, settings);
            setPreparing(null);
          }}
        />
      ) : null}

      {configured ? (
        <WidgetSettings
          title={t("dashboard.settings.title", { widget: name(configured.placement.widget) })}
          icon={iconOf(configured.placement.widget)}
          declared={settingsOf(declared(configured.placement.widget))}
          values={configured.placement.settings}
          onClose={() => setConfiguring(null)}
          onApply={(settings) => {
            change((d) => d.map((x) => (x.key === configured.key ? { ...x, placement: { ...x.placement, settings }, node: null } : x)));
            setConfiguring(null);
          }}
        />
      ) : null}

      <ConfirmDialog
        open={resetting}
        title={t("dashboard.reset.title")}
        confirmLabel={t("dashboard.reset")}
        cancelLabel={t("dashboard.cancel")}
        busy={busy}
        onConfirm={() => void reset()}
        onClose={() => setResetting(false)}
      >
        {t("dashboard.reset.body")}
      </ConfirmDialog>
      {guard}
    </>
  );
}
