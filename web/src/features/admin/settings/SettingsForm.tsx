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
 * The settings of a scope, grouped by the module that declares them and found by words of their name, key or module.
 * A change waits in the floating bar until it is saved; each is saved on the version it was read on, and a page left
 * with changes asks first.
 */

import InputAdornment from "@mui/material/InputAdornment";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { ChangesBar, Icon, SectionCard, TextField, useToast } from "@kampus/ui";

import { browserApi } from "../../../api/browser";
import type { ScopeSetting } from "../../../api/types";
import { useI18n } from "../../../i18n/I18nProvider";
import { EmptyState } from "../../shell/EmptyState";
import { useLeaveGuard } from "../../shell/LeaveGuard";
import { moduleName, settingName } from "../names";
import { type Draft, SettingRow, problemOf } from "./SettingRow";

/** The icon of the settings of a module. */
const ICONS: Readonly<Record<string, string>> = {
  identity: "shield_person",
  courses: "school",
  notify: "notifications",
  progress: "trending_up",
};

/** The props of `SettingsForm`. */
export interface SettingsFormProps {
  /** The scope the settings are of. */
  readonly scope: string;
  readonly settings: readonly ScopeSetting[];
}

/** `<SettingsForm scope={campus} settings={settings} />` */
export function SettingsForm({ scope, settings }: SettingsFormProps) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [drafts, setDrafts] = useState<ReadonlyMap<string, Draft>>(new Map());
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const guard = useLeaveGuard(drafts.size > 0);

  const groups = useMemo(() => {
    const words = query.trim().toLowerCase();
    const found = settings.filter(
      (s) => !words || [settingName(t, s), s.key, moduleName(t, s.module)].some((text) => text.toLowerCase().includes(words)),
    );
    const byModule = new Map<string, ScopeSetting[]>();
    for (const setting of found) byModule.set(setting.module, [...(byModule.get(setting.module) ?? []), setting]);
    return [...byModule.entries()].sort(([a], [b]) => moduleName(t, a).localeCompare(moduleName(t, b)));
  }, [settings, query, t]);

  const change = (setting: ScopeSetting, draft: Draft) => {
    setDrafts((current) => {
      const next = new Map(current);
      const same = JSON.stringify(draft.value) === JSON.stringify(setting.value) && draft.locked === setting.locked;
      if (same) next.delete(setting.key);
      else next.set(setting.key, draft);
      return next;
    });
  };
  const invalid = settings.some((s) => drafts.has(s.key) && problemOf(s, drafts.get(s.key)!.value) !== null);

  const path = (key: string) => `/settings/scopes/${scope}/${encodeURIComponent(key)}`;

  async function save() {
    setBusy(true);
    const failed: string[] = [];
    for (const setting of settings) {
      const draft = drafts.get(setting.key);
      if (!draft) continue;
      try {
        await browserApi<ScopeSetting>(path(setting.key), {
          method: "PUT",
          headers: { "If-Match": `"${setting.version}"` },
          body: { value: draft.value, locked: draft.locked },
        });
      } catch {
        failed.push(setting.key);
      }
    }
    setDrafts((current) => new Map([...current].filter(([key]) => failed.includes(key))));
    setBusy(false);
    toast(failed.length ? t("admin.failed", { n: failed.length }) : t("admin.saved"), failed.length ? "danger" : "success");
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

  if (settings.length === 0) return <EmptyState icon="tune" message={t("admin.empty")} />;
  return (
    <Stack spacing={3}>
      <Typography color="text.secondary">{t("admin.section.settings.lead")}</Typography>
      <TextField
        label={t("admin.search")}
        type="search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        startAdornment={
          <InputAdornment position="start">
            <Icon name="search" size="small" />
          </InputAdornment>
        }
        sx={{ maxInlineSize: 420 }}
      />
      {groups.length === 0 ? (
        <Typography color="text.secondary" role="status">
          {t("admin.search.none", { q: query.trim() })}
        </Typography>
      ) : (
        groups.map(([module, items]) => (
          <SectionCard key={module} title={moduleName(t, module)} icon={ICONS[module] ?? "settings"}>
            {items.map((setting) => (
              <SettingRow
                key={setting.key}
                setting={setting}
                scope={scope}
                draft={drafts.get(setting.key)}
                busy={busy}
                onChange={(draft) => change(setting, draft)}
                onReset={() => void reset(setting)}
              />
            ))}
          </SectionCard>
        ))
      )}
      <ChangesBar
        open={drafts.size > 0}
        label={t("admin.section.settings")}
        message={t("admin.changes", { n: drafts.size })}
        saveLabel={t("admin.save")}
        discardLabel={t("changes.discard")}
        busy={busy}
        onSave={() => {
          if (invalid) toast(t("admin.invalid"), "danger");
          else void save();
        }}
        onDiscard={() => setDrafts(new Map())}
      />
      {guard}
    </Stack>
  );
}
