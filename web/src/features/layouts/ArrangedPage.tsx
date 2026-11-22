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
 * home of a campus or the page of a course. A person who may arrange it gets "Customise page": every widget then shows
 * its controls (move it up, down or to another region, change its settings, remove it), each region offers to add a
 * widget, and the arrangement is saved for the scope, or put back to the default.
 */

import Alert from "@mui/material/Alert";
import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardActionArea from "@mui/material/CardActionArea";
import Dialog from "@mui/material/Dialog";
import DialogContent from "@mui/material/DialogContent";
import DialogTitle from "@mui/material/DialogTitle";
import Grid from "@mui/material/Grid";
import IconButton from "@mui/material/IconButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import Stack from "@mui/material/Stack";
import Tooltip from "@mui/material/Tooltip";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useState, type ReactNode } from "react";

import { Button, ConfirmDialog, Icon, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import type { AvailableWidget, DeclaredSetting, Placement } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
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
  /** What the bar of the editor says: who sees what is saved. */
  readonly editingText: string;
}

let keys = 0;

/** The settings a widget declares, as the API sends them. */
const settingsOf = (widget: AvailableWidget | undefined): DeclaredSetting[] => (widget?.settings ?? []) as DeclaredSetting[];

/** `<ArrangedPage scope="…" pageKind="customise.dashboard" regions={["top", "main", "side"]} widgets={…} … />` */
export function ArrangedPage({ scope, pageKind, regions, widgets, editable, available, editingText }: ArrangedPageProps) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState<Draft[]>([]);
  const [busy, setBusy] = useState(false);
  const [adding, setAdding] = useState<string | null>(null);
  const [configuring, setConfiguring] = useState<number | null>(null);
  const [resetting, setResetting] = useState(false);

  const shown: Draft[] = editing ? draft : widgets.map((w) => ({ key: 0, placement: w.placement, node: w.node }));
  const declared = (code: string) => available.find((w) => w.code === code);
  const name = (code: string) => t(`widget.${code}`) === `widget.${code}` ? code : t(`widget.${code}`);

  function begin() {
    setDraft(widgets.map((w) => ({ key: ++keys, placement: w.placement, node: w.node })));
    setEditing(true);
  }

  function move(key: number, region: string, by: number) {
    setDraft((items) => {
      const at = items.findIndex((d) => d.key === key);
      if (at < 0) return items;
      const item = items[at]!;
      const rest = items.filter((d) => d.key !== key);
      if (region !== item.placement.region) return [...rest, { ...item, placement: { ...item.placement, region } }];
      const siblings = rest.filter((d) => d.placement.region === region);
      const index = items.filter((d) => d.placement.region === region).findIndex((d) => d.key === key) + by;
      const before = siblings[index];
      const position = before ? rest.indexOf(before) : rest.length;
      return [...rest.slice(0, position), item, ...rest.slice(position)];
    });
  }

  async function save() {
    setBusy(true);
    try {
      await browserApi(`/customise/scopes/${scope}/layouts/${pageKind}`, {
        method: "PUT",
        body: { placements: draft.map((d) => ({ region: d.placement.region, widget: d.placement.widget, settings: d.placement.settings })) },
      });
      toast(t("dashboard.saved"), "success");
      setEditing(false);
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  async function reset() {
    setBusy(true);
    try {
      await browserApi(`/customise/scopes/${scope}/layouts/${pageKind}`, { method: "DELETE" });
      toast(t("dashboard.restored"), "success");
      setEditing(false);
      setResetting(false);
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  const sideUsed = editing || shown.some((d) => d.placement.region === "side");
  const span = (region: string) => (region === "main" ? (sideUsed ? 8 : 12) : region === "side" ? 4 : 12);
  const configured = draft.find((d) => d.key === configuring);

  return (
    <>
      {editable && !editing ? (
        <Box sx={{ display: "flex", justifyContent: "flex-end", mb: 2 }}>
          <Button variant="tonal" icon="space_dashboard" onClick={begin}>
            {t("dashboard.customise")}
          </Button>
        </Box>
      ) : null}
      {editing ? (
        <Alert
          severity="info"
          icon={<Icon name="edit_square" />}
          sx={{ mb: 3, alignItems: "center", position: "sticky", top: 72, zIndex: "appBar", boxShadow: 3 }}
          action={
            <Stack direction="row" spacing={1}>
              <Button variant="text" tone="danger" icon="restart_alt" onClick={() => setResetting(true)} disabled={busy}>
                {t("dashboard.reset")}
              </Button>
              <Button variant="outlined" onClick={() => setEditing(false)} disabled={busy}>
                {t("dashboard.cancel")}
              </Button>
              <Button variant="filled" icon="save" busy={busy} onClick={() => void save()}>
                {t("dashboard.save")}
              </Button>
            </Stack>
          }
        >
          {editingText}
        </Alert>
      ) : null}
      <Grid container spacing={3}>
        {regions.map((region) => {
          const items = shown.filter((d) => d.placement.region === region);
          if (!editing && items.length === 0) return null;
          return (
            <Grid key={region} size={{ xs: 12, md: span(region) }}>
              <Stack
                spacing={3}
                sx={
                  editing
                    ? { p: 2, borderRadius: 4, border: "2px dashed", borderColor: "primary.main", bgcolor: "color-mix(in srgb, var(--k-palette-primary-main) 4%, transparent)" }
                    : undefined
                }
              >
                {editing ? (
                  <Typography variant="overline" color="primary">
                    {t(`region.${region}`)}
                  </Typography>
                ) : null}
                {items.map((item, at) =>
                  editing ? (
                    <Card key={item.key} sx={{ overflow: "visible", boxShadow: 2 }}>
                      <Stack direction="row" spacing={1} sx={{ alignItems: "center", px: 1.5, py: 1, borderBottom: 1, borderColor: "divider" }}>
                        <Box sx={{ color: "primary.main", display: "flex" }}>
                          <Icon name={WIDGET_ICONS[item.placement.widget] ?? "widgets"} size="small" />
                        </Box>
                        <Typography variant="subtitle2" sx={{ flex: 1 }} noWrap>
                          {name(item.placement.widget)}
                        </Typography>
                        <Tooltip title={t("dashboard.moveup")}>
                          <span>
                            <IconButton size="small" aria-label={t("dashboard.moveup")} disabled={at === 0} onClick={() => move(item.key, region, -1)}>
                              <Icon name="arrow_upward" size="small" />
                            </IconButton>
                          </span>
                        </Tooltip>
                        <Tooltip title={t("dashboard.movedown")}>
                          <span>
                            <IconButton size="small" aria-label={t("dashboard.movedown")} disabled={at === items.length - 1} onClick={() => move(item.key, region, 1)}>
                              <Icon name="arrow_downward" size="small" />
                            </IconButton>
                          </span>
                        </Tooltip>
                        <RegionMenu regions={regions.filter((r) => r !== region)} onMove={(to) => move(item.key, to, 0)} />
                        {settingsOf(declared(item.placement.widget)).length > 0 ? (
                          <Tooltip title={t("dashboard.settings")}>
                            <IconButton size="small" aria-label={t("dashboard.settings")} onClick={() => setConfiguring(item.key)}>
                              <Icon name="tune" size="small" />
                            </IconButton>
                          </Tooltip>
                        ) : null}
                        <Tooltip title={t("dashboard.remove")}>
                          <IconButton size="small" aria-label={t("dashboard.remove")} sx={{ color: "error.main" }} onClick={() => setDraft((d) => d.filter((x) => x.key !== item.key))}>
                            <Icon name="delete" size="small" />
                          </IconButton>
                        </Tooltip>
                      </Stack>
                      <Box sx={{ p: 2, pointerEvents: "none", opacity: 0.92, maxBlockSize: 420, overflow: "hidden" }} inert>
                        {item.node ?? (
                          <Stack spacing={1} sx={{ alignItems: "center", py: 4, color: "text.secondary" }}>
                            <Icon name={WIDGET_ICONS[item.placement.widget] ?? "widgets"} size="large" />
                            <Typography variant="body2">{t("dashboard.preview")}</Typography>
                          </Stack>
                        )}
                      </Box>
                    </Card>
                  ) : (
                    <Box key={`${item.placement.widget}-${at}`}>{item.node}</Box>
                  ),
                )}
                {editing ? (
                  <Button variant="outlined" icon="add_circle" onClick={() => setAdding(region)} sx={{ alignSelf: "center", borderStyle: "dashed" }}>
                    {t("dashboard.add")}
                  </Button>
                ) : null}
              </Stack>
            </Grid>
          );
        })}
      </Grid>

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
                      setDraft((d) => [...d, { key: ++keys, placement: { region: adding ?? "main", widget: widget.code, settings, permission: null }, node: null }]);
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
            setDraft((d) => d.map((x) => (x.key === configured.key ? { ...x, placement: { ...x.placement, settings }, node: null } : x)));
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
    </>
  );
}

/** The menu that moves a widget to another region. */
function RegionMenu({ regions, onMove }: { readonly regions: readonly string[]; readonly onMove: (region: string) => void }) {
  const { t } = useI18n();
  const [anchor, setAnchor] = useState<HTMLElement | null>(null);
  return (
    <>
      <Tooltip title={t("dashboard.moveto", { region: "…" })}>
        <IconButton size="small" aria-label={t("dashboard.moveto", { region: "…" })} aria-haspopup="menu" onClick={(e) => setAnchor(e.currentTarget)}>
          <Icon name="drag_indicator" size="small" />
        </IconButton>
      </Tooltip>
      <Menu anchorEl={anchor} open={Boolean(anchor)} onClose={() => setAnchor(null)}>
        {regions.map((region) => (
          <MenuItem
            key={region}
            onClick={() => {
              onMove(region);
              setAnchor(null);
            }}
          >
            <ListItemIcon>
              <Icon name="arrow_forward" size="small" />
            </ListItemIcon>
            {t("dashboard.moveto", { region: t(`region.${region}`) })}
          </MenuItem>
        ))}
      </Menu>
    </>
  );
}
