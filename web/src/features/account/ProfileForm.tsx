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
 * The profile of the person: their names, how their name is shown, their language and their time zone. It is saved
 * as a whole against the version read, so a change made elsewhere meanwhile is never overwritten.
 */

import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Grid from "@mui/material/Grid";
import Stack from "@mui/material/Stack";
import { useRouter } from "next/navigation";
import { useMemo, useState, type FormEvent } from "react";

import { Button, SelectField, TextField, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import type { Me, Profile } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { LANGUAGES } from "../../i18n/translator";

/** The props of `ProfileForm`. */
export interface ProfileFormProps {
  readonly account: Me;
  readonly profile: Profile;
}

/** `<ProfileForm account={…} profile={…} />` */
export function ProfileForm({ account, profile }: ProfileFormProps) {
  const { t } = useI18n();
  const router = useRouter();
  const toast = useToast();
  const [version, setVersion] = useState(profile.version);
  const [busy, setBusy] = useState(false);
  const zones = useMemo(() => Intl.supportedValuesOf("timeZone"), []);

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const value = (name: string) => {
      const text = String(form.get(name) ?? "").trim();
      return text === "" ? null : text;
    };
    setBusy(true);
    try {
      const saved = await browserApi<Profile>("/users/me/profile", {
        method: "PUT",
        headers: { "If-Match": `"${version}"` },
        body: {
          given_name: value("given_name"),
          family_name: value("family_name"),
          display_name: value("display_name"),
          language_code: value("language_code"),
          time_zone: value("time_zone"),
        },
      });
      setVersion(saved.version);
      toast(t("profile.saved"), "success");
      router.refresh();
    } catch (failure) {
      toast(failure instanceof ApiError && failure.status === 412 ? t("profile.conflict") : t("error.generic"), "danger");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Card>
      <CardContent sx={{ p: { xs: 2.5, md: 4 } }}>
        <Stack component="form" spacing={3} onSubmit={(event) => void save(event)}>
          <Grid container spacing={2.5}>
            <Grid size={{ xs: 12, md: 6 }}>
              <TextField label={t("profile.username")} value={account.username} disabled />
            </Grid>
            <Grid size={{ xs: 12, md: 6 }}>
              <TextField label={t("profile.email")} value={account.email} disabled />
            </Grid>
            <Grid size={{ xs: 12, md: 6 }}>
              <TextField name="given_name" label={t("profile.given")} defaultValue={profile.given_name ?? ""} autoComplete="given-name" />
            </Grid>
            <Grid size={{ xs: 12, md: 6 }}>
              <TextField name="family_name" label={t("profile.family")} defaultValue={profile.family_name ?? ""} autoComplete="family-name" />
            </Grid>
            <Grid size={12}>
              <TextField name="display_name" label={t("profile.display")} hint={t("profile.display.hint")} defaultValue={profile.display_name ?? ""} autoComplete="nickname" />
            </Grid>
            <Grid size={{ xs: 12, md: 6 }}>
              <SelectField
                name="language_code"
                label={t("profile.language")}
                defaultValue={profile.language_code ?? ""}
                options={[{ value: "", label: t("profile.language.auto") }, ...LANGUAGES.map((l) => ({ value: l, label: t(`language.${l}`) === `language.${l}` ? l : t(`language.${l}`) }))]}
              />
            </Grid>
            <Grid size={{ xs: 12, md: 6 }}>
              <SelectField
                name="time_zone"
                label={t("profile.timezone")}
                searchPlaceholder={t("select.search")}
                noMatch={t("select.nomatch")}
                defaultValue={profile.time_zone ?? ""}
                options={[{ value: "", label: t("profile.timezone.auto") }, ...zones.map((z) => ({ value: z, label: z.replaceAll("_", " ") }))]}
              />
            </Grid>
          </Grid>
          <Stack direction="row" sx={{ justifyContent: "flex-end" }}>
            <Button type="submit" variant="filled" icon="save" busy={busy}>
              {t("profile.save")}
            </Button>
          </Stack>
        </Stack>
      </CardContent>
    </Card>
  );
}
