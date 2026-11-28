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
 * The settings of one module at one level (the campus, or the platform for those set there), in the sections of their
 * groups. Only the changed ones can be shown; changes wait in the floating bar until they are saved, each on the version
 * it was read on, and leaving the page with changes asks first.
 */

import Stack from "@mui/material/Stack";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { ChangesBar, Icon, IconTile, SectionCard, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import { ApiError } from "../../../api/problem";
import type { ScopeSetting } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { PageHeader } from "../../shell/PageHeader";
import { useLeaveGuard } from "../../shell/LeaveGuard";
import { MODULE_ICONS } from "../sections";
import { problemOf, same } from "./format";
import { HistoryDialog } from "./HistoryDialog";
import { type Draft, SettingItem } from "./SettingItem";

/** A level the settings of a page may be set at. */
export interface Level {
  readonly kind: "campus" | "platform";
  readonly scope: string;
}

/** The props of `ModuleSettings`. */
export interface ModuleSettingsProps {
  readonly module: string;
  /** The level shown. */
  readonly level: Level;
  /** The levels the person may set the settings of this module at. */
  readonly levels: readonly Level["kind"][];
  readonly settings: readonly ScopeSetting[];
}

/** `<ModuleSettings module="identity" level={campus} levels={["campus", "platform"]} settings={settings} />` */
export function ModuleSettings({ module, level, levels, settings }: ModuleSettingsProps) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [drafts, setDrafts] = useState<ReadonlyMap<string, Draft>>(new Map());
  const [failed, setFailed] = useState<ReadonlyMap<string, string>>(new Map());
  const [busy, setBusy] = useState(false);
  const [history, setHistory] = useState<ScopeSetting | null>(null);
  const guard = useLeaveGuard(drafts.size > 0);

  const groups = useMemo(() => {
    const byGroup = new Map<string, ScopeSetting[]>();
    for (const setting of settings) byGroup.set(setting.group, [...(byGroup.get(setting.group) ?? []), setting]);
    return [...byGroup.entries()];
  }, [settings]);
  const changedCount = settings.filter((s) => !same(s.value, s.default_value)).length;

  const change = (setting: ScopeSetting, draft: Draft) =>
    setDrafts((current) => {
      const next = new Map(current);
      if (same(draft.value, setting.value) && draft.locked === setting.locked) next.delete(setting.key);
      else next.set(setting.key, draft);
      return next;
    });
  const invalid = settings.some((s) => drafts.has(s.key) && problemOf(s, drafts.get(s.key)!.value) !== null);
  const path = (key: string) => `/settings/scopes/${level.scope}/${encodeURIComponent(key)}`;

  async function save() {
    if (invalid) {
      toast(t("admin.invalid"), "danger");
      return;
    }
    setBusy(true);
    const problems = new Map<string, string>();
    for (const setting of settings) {
      const draft = drafts.get(setting.key);
      if (!draft) continue;
      try {
        await browserApi<ScopeSetting>(path(setting.key), {
          method: "PUT",
          headers: { "If-Match": `"${setting.version}"` },
          body: { value: draft.value, locked: draft.locked },
        });
      } catch (error) {
        problems.set(setting.key, error instanceof ApiError ? error.status.toString() : "error");
      }
    }
    setFailed(problems);
    setDrafts((current) => new Map([...current].filter(([key]) => problems.has(key))));
    setBusy(false);
    toast(problems.size ? t("admin.failed", { n: problems.size }) : t("admin.saved"), problems.size ? "danger" : "success");
    router.refresh();
  }

  async function reset(setting: ScopeSetting) {
    setBusy(true);
    try {
      await browserApi<ScopeSetting>(path(setting.key), { method: "DELETE" });
      setDrafts((current) => new Map([...current].filter(([key]) => key !== setting.key)));
      toast(t("admin.reset.done"), "success");
      router.refresh();
    } catch {
      toast(t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  const choose = (kind: Level["kind"]) => {
    if (kind !== level.kind) router.push(kind === "campus" ? "?" : `?level=${kind}`);
  };

  return (
    <Stack spacing={3}>
      <PageHeader
        title={t(`admin.module.${module}`)}
        lead={t(`admin.module.${module}.lead`) === `admin.module.${module}.lead` ? null : t(`admin.module.${module}.lead`)}
        leading={<IconTile name={MODULE_ICONS[module] ?? "tune"} size="large" />}
        actions={
          levels.length > 1 ? (
            <ToggleButtonGroup
              exclusive
              size="small"
              value={level.kind}
              aria-label={t("admin.level")}
              onChange={(_, next: Level["kind"] | null) => next && choose(next)}
              sx={{ "& .MuiToggleButton-root": { textTransform: "none", px: 2, fontWeight: 600, gap: 1 } }}
            >
              <ToggleButton value="campus">
                <Icon name="school" size="small" />
                {t("admin.level.campus")}
              </ToggleButton>
              <ToggleButton value="platform">
                <Icon name="public" size="small" />
                {t("admin.level.platform")}
              </ToggleButton>
            </ToggleButtonGroup>
          ) : undefined
        }
      />
      <Stack direction="row" sx={{ alignItems: "center", justifyContent: "space-between", flexWrap: "wrap", gap: 1, mt: -1 }}>
        <Typography variant="body2" color="text.secondary">
          {level.kind === "platform" ? t("admin.level.platform.lead") : changedCount ? t("admin.count.changed", { n: changedCount }) : t("admin.count.none")}
        </Typography>
      </Stack>
      {groups.map(([group, items]) => (
          <SectionCard key={group} title={t(`${module}.group.${group}`) === `${module}.group.${group}` ? t("admin.group.general") : t(`${module}.group.${group}`)}>
            {items.map((setting) => (
              <SettingItem
                key={setting.key}
                setting={setting}
                scope={level.scope}
                draft={drafts.get(setting.key)}
                failed={failed.get(setting.key)}
                busy={busy}
                onChange={(draft) => change(setting, draft)}
                onReset={() => void reset(setting)}
                onHistory={() => setHistory(setting)}
              />
            ))}
          </SectionCard>
        ))}
      <ChangesBar
        open={drafts.size > 0}
        label={t(`admin.module.${module}`)}
        message={t("admin.changes", { n: drafts.size })}
        saveLabel={t("admin.save")}
        discardLabel={t("changes.discard")}
        busy={busy}
        onSave={() => void save()}
        onDiscard={() => {
          setDrafts(new Map());
          setFailed(new Map());
        }}
      />
      <HistoryDialog scope={level.scope} setting={history} onClose={() => setHistory(null)} />
      {guard}
    </Stack>
  );
}
