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
 * A page arranged with widgets in its regions (top across the page, then the main column and the side), such as the
 * home of a campus or the page of a course. For a person who may arrange it, the edit mode of the bar turns it into
 * its editor: widgets are dragged within and between regions, their settings open at the side, each region offers to
 * add one, and the arrangement can go back to the default. Every change is saved as it is made.
 */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardActionArea from "@mui/material/CardActionArea";
import Dialog from "@mui/material/Dialog";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Grid from "@mui/material/Grid";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  TouchSensor,
  closestCenter,
  useSensor,
  useSensors,
  type Announcements,
  type UniqueIdentifier,
} from "@dnd-kit/core";
import { arrayMove, sortableKeyboardCoordinates } from "@dnd-kit/sortable";
import { useRouter } from "next/navigation";
import Tooltip from "@mui/material/Tooltip";
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";

import { Button, ConfirmDialog, Icon, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { AvailableWidget, DeclaredSetting, Placement } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { useOfferEditMode } from "../shell/EditMode";
import { useLeaveGuard } from "../shell/LeaveGuard";
import { DropRegion } from "./DropRegion";
import { SortableWidget } from "./SortableWidget";
import { WidgetFrame } from "./WidgetFrame";
import { WidgetSettings } from "./WidgetSettings";
import { WIDGET_ICONS } from "./widget-icons";

/** A widget placed on the page, with what it shows when it was rendered by the server. */
export interface PlacedWidget {
  readonly placement: Placement;
  readonly node: ReactNode;
}

/** A widget while the page is arranged; one added since has nothing to show until it is saved. */
interface Draft {
  readonly key: number;
  readonly placement: Placement;
  readonly node: ReactNode | null;
}

/** The props of `ArrangedPage`. */
export interface ArrangedPageProps {
  /** The scope the page is arranged for. */
  readonly scope: string;
  /** The page kind. */
  readonly pageKind: string;
  /** Its regions, in order. */
  readonly regions: readonly string[];
  /** Its widgets, as rendered. */
  readonly widgets: readonly PlacedWidget[];
  /** Whether the reader may arrange it. */
  readonly editable: boolean;
  /** The widgets the reader may add. */
  readonly available: readonly AvailableWidget[];
}

let keys = 0;

/** The settings a widget declares, as the API sends them. */
const settingsOf = (widget: AvailableWidget | undefined): DeclaredSetting[] => (widget?.settings ?? []) as DeclaredSetting[];

/** `<ArrangedPage scope="…" pageKind="customise.dashboard" regions={["top", "main", "side"]} widgets={…} … />` */
export function ArrangedPage({ scope, pageKind, regions, widgets, editable, available }: ArrangedPageProps) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const editing = useOfferEditMode(editable);
  const [draft, setDraft] = useState<Draft[]>([]);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState(false);
  const [adding, setAdding] = useState<string | null>(null);
  const [configuring, setConfiguring] = useState<number | null>(null);
  const [resetting, setResetting] = useState(false);
  const [selected, setSelected] = useState<number | null>(null);
  const [dragged, setDragged] = useState<number | null>(null);
  const [changed, setChanged] = useState(0);
  const justMoved = useRef(false);
  const latest = useRef<Draft[]>([]);
  latest.current = draft;
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 180, tolerance: 6 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const declared = (code: string) => available.find((w) => w.code === code);
  const name = (code: string) => (t(`widget.${code}`) === `widget.${code}` ? code : t(`widget.${code}`));
  const fresh = () => widgets.map((w) => ({ key: ++keys, placement: w.placement, node: w.node }));

  // The arrangement starts from the page as rendered, and follows it again after each save that brought new content.
  useEffect(() => {
    if (editing && !pending && dragged === null) setDraft(fresh());
  }, [editing, widgets]);

  /** Saves the arrangement as it is now; content is asked again only when a widget needs it. */
  const save = useCallback(async () => {
    const items = latest.current;
    setPending(false);
    try {
      await browserApi(`/customise/scopes/${scope}/layouts/${pageKind}`, {
        method: "PUT",
        body: { placements: items.map((d) => ({ region: d.placement.region, widget: d.placement.widget, settings: d.placement.settings })) },
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

  // Leaving the mode saves what is still pending at once.
  const wasEditing = useRef(editing);
  useEffect(() => {
    if (wasEditing.current && !editing && pending) void save();
    wasEditing.current = editing;
  }, [editing, pending, save]);

  /** Records a change of the person, to be saved. */
  const change = (update: (items: Draft[]) => Draft[]) => {
    setDraft(update);
    setChanged((n) => n + 1);
  };

  const nameOf = (id: UniqueIdentifier) => name(draft.find((d) => d.key === Number(id))?.placement.widget ?? "");
  /** The region of a widget, or the region a drop target is. */
  const regionOf = (id: UniqueIdentifier): string | undefined =>
    typeof id === "string" && id.startsWith("region:") ? id.slice("region:".length) : draft.find((d) => d.key === Number(id))?.placement.region;
  const placeOf = (id: UniqueIdentifier | undefined) => (id === undefined ? "" : t(`region.${regionOf(id) ?? ""}`));
  const announcements: Announcements = {
    onDragStart: ({ active }) => t("drag.start", { widget: nameOf(active.id) }),
    onDragOver: ({ active, over }) => (over ? t("drag.over", { widget: nameOf(active.id), region: placeOf(over.id) }) : undefined),
    onDragEnd: ({ active, over }) => (over ? t("drag.end", { widget: nameOf(active.id), region: placeOf(over.id) }) : t("drag.cancel", { widget: nameOf(active.id) })),
    onDragCancel: ({ active }) => t("drag.cancel", { widget: nameOf(active.id) }),
  };

  /**
   * A widget dragged over another region joins it, before the widget it is over or at its end. After a move, the
   * next ones wait for the next frame: the region it left is then the nearest target, and moving back at once would
   * swing the widget between both for ever.
   */
  function moveOver(key: number, over: UniqueIdentifier) {
    if (justMoved.current) return;
    setDraft((items) => {
      const item = items.find((d) => d.key === key);
      const to = typeof over === "string" && over.startsWith("region:") ? over.slice("region:".length) : items.find((d) => d.key === over)?.placement.region;
      if (!to || !item || item.placement.region === to) return items;
      justMoved.current = true;
      requestAnimationFrame(() => {
        justMoved.current = false;
      });
      const rest = items.filter((d) => d.key !== key);
      const moved = { ...item, placement: { ...item.placement, region: to } };
      const at = typeof over === "number" ? rest.findIndex((d) => d.key === over) : -1;
      if (at >= 0) return [...rest.slice(0, at), moved, ...rest.slice(at)];
      const last = rest.map((d) => d.placement.region).lastIndexOf(to);
      return [...rest.slice(0, last + 1), moved, ...rest.slice(last + 1)];
    });
  }

  /** A widget dropped on another of its region takes its place; any drop is a change to save. */
  function drop(key: number, over: UniqueIdentifier | undefined) {
    change((items) => {
      if (typeof over !== "number" || over === key) return items;
      const from = items.findIndex((d) => d.key === key);
      const to = items.findIndex((d) => d.key === over);
      if (from < 0 || to < 0 || items[from]!.placement.region !== items[to]!.placement.region) return items;
      return arrayMove(items, from, to);
    });
  }

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

  const shown: Draft[] = editing ? draft : widgets.map((w) => ({ key: 0, placement: w.placement, node: w.node }));
  const lifted = dragged === null ? undefined : draft.find((d) => d.key === dragged);
  const guard = useLeaveGuard(pending);
  const sideUsed = editing || shown.some((d) => d.placement.region === "side");
  const span = (region: string) => (region === "main" ? (sideUsed ? 8 : 12) : region === "side" ? 4 : 12);
  const configured = draft.find((d) => d.key === configuring);
  const preview = (code: string) => (
    <Stack spacing={1} sx={{ alignItems: "center", py: 4, color: "text.secondary" }}>
      <Icon name={WIDGET_ICONS[code] ?? "widgets"} size="large" />
      <Typography variant="body2">{t("dashboard.preview")}</Typography>
    </Stack>
  );

  return (
    <>
      {editing ? (
        <Stack direction="row" spacing={1} sx={{ justifyContent: "flex-end", mb: 2 }}>
          <Tooltip title={t("dashboard.reset.hint")}>
            <span>
              <Button variant="tonal" icon="restart_alt" onClick={() => setResetting(true)} disabled={busy}>
                {t("dashboard.reset")}
              </Button>
            </span>
          </Tooltip>
        </Stack>
      ) : null}
      {editing ? (
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          accessibility={{ announcements, screenReaderInstructions: { draggable: t("drag.instructions") } }}
          onDragStart={({ active }) => setDragged(Number(active.id))}
          onDragOver={({ active, over }) => over && moveOver(Number(active.id), over.id)}
          onDragEnd={({ active, over }) => {
            setDragged(null);
            drop(Number(active.id), over?.id);
          }}
          onDragCancel={() => setDragged(null)}
        >
          <Grid container spacing={3}>
            {regions.map((region) => {
              const items = shown.filter((d) => d.placement.region === region);
              return (
                <Grid key={region} size={{ xs: 12, md: span(region) }}>
                  <DropRegion region={region} label={t(`region.${region}`)} items={items.map((d) => d.key)}>
                    {items.map((item) => (
                      <SortableWidget
                        key={item.key}
                        id={item.key}
                        name={name(item.placement.widget)}
                        icon={WIDGET_ICONS[item.placement.widget] ?? "widgets"}
                        selected={selected === item.key}
                        onSelect={() => setSelected(item.key)}
                        onSettings={settingsOf(declared(item.placement.widget)).length > 0 ? () => setConfiguring(item.key) : undefined}
                        onRemove={() => change((d) => d.filter((x) => x.key !== item.key))}
                      >
                        {item.node ?? preview(item.placement.widget)}
                      </SortableWidget>
                    ))}
                    <Button variant="outlined" icon="add_circle" onClick={() => setAdding(region)} sx={{ alignSelf: "center", borderStyle: "dashed" }}>
                      {t("dashboard.add")}
                    </Button>
                  </DropRegion>
                </Grid>
              );
            })}
          </Grid>
          <DragOverlay dropAnimation={{ duration: 180, easing: "cubic-bezier(0.2, 0, 0, 1)" }}>
            {lifted ? (
              <WidgetFrame lifted name={name(lifted.placement.widget)} icon={WIDGET_ICONS[lifted.placement.widget] ?? "widgets"}>
                {lifted.node ?? preview(lifted.placement.widget)}
              </WidgetFrame>
            ) : null}
          </DragOverlay>
        </DndContext>
      ) : (
        <Grid container spacing={3}>
          {regions.map((region) => {
            const items = shown.filter((d) => d.placement.region === region);
            if (items.length === 0) return null;
            return (
              <Grid key={region} size={{ xs: 12, md: span(region) }}>
                <Stack spacing={3}>
                  {items.map((item, at) => (
                    <Box key={`${item.placement.widget}-${at}`}>{item.node}</Box>
                  ))}
                </Stack>
              </Grid>
            );
          })}
        </Grid>
      )}

      <Dialog open={adding !== null} onClose={() => setAdding(null)} maxWidth="sm" fullWidth aria-labelledby="add-widget">
        <DialogTitle id="add-widget">{t("dashboard.add.title", { region: adding ? t(`region.${adding}`) : "" })}</DialogTitle>
        <DialogContent>
          <Grid container spacing={1.5}>
            {available.map((widget) => (
              <Grid key={widget.code} size={{ xs: 12, sm: 6 }}>
                <Card sx={{ blockSize: "100%" }}>
                  <CardActionArea
                    sx={{ p: 2, blockSize: "100%", display: "flex", alignItems: "flex-start", justifyContent: "flex-start", gap: 1.5 }}
                    onClick={() => {
                      const settings = Object.fromEntries(settingsOf(widget).map((s) => [s.key, s.default]));
                      change((d) => [...d, { key: ++keys, placement: { region: adding ?? "main", widget: widget.code, settings, permission: null }, node: null }]);
                      setAdding(null);
                    }}
                  >
                    <Box sx={{ display: "grid", placeItems: "center", inlineSize: 40, blockSize: 40, flex: "none", borderRadius: 2.5, backgroundImage: "var(--k-palette-gradient-soft)", color: "primary.main" }}>
                      <Icon name={WIDGET_ICONS[widget.code] ?? "widgets"} />
                    </Box>
                    <Box>
                      <Typography variant="subtitle2">{name(widget.code)}</Typography>
                      <Typography variant="caption" color="text.secondary">
                        {t(`about.${widget.code}`) === `about.${widget.code}` ? widget.summary : t(`about.${widget.code}`)}
                      </Typography>
                    </Box>
                  </CardActionArea>
                </Card>
              </Grid>
            ))}
          </Grid>
        </DialogContent>
      </Dialog>

      {configured ? (
        <WidgetSettings
          title={t("dashboard.settings.title", { widget: name(configured.placement.widget) })}
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
