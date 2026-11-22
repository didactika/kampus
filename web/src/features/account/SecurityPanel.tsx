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

/** The security of the account: changing the password, its second factors, and the sessions open on other devices, each of which can be closed. */

import Box from "@mui/material/Box";
import Card from "@mui/material/Card";
import CardContent from "@mui/material/CardContent";
import Stack from "@mui/material/Stack";
import Typography from "@mui/material/Typography";
import { useRouter } from "next/navigation";
import { useState } from "react";

import { Button, Icon, useToast } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import type { SecondFactor, SessionSummary } from "../../api/types";
import { useI18n } from "../../i18n/I18nProvider";
import { EmptyState } from "../shell/EmptyState";
import { NewPasswordForm } from "../session/NewPasswordForm";
import { PasswordField } from "../session/PasswordField";
import { deviceOf } from "./device";
import { SecondFactors } from "./SecondFactors";
import { StepUpProvider } from "./StepUp";

/** How many sessions show before the rest are asked for. */
const SHOWN = 6;

/**
 * A form that changes the password, asking for the current one and the new one twice. When the session is too old for
 * a sensitive change, the current password the person just typed raises it first, so they are never asked twice. The
 * change ends every session, this one too, so the person signs in again with the new password.
 */
function PasswordForm() {
  const { t } = useI18n();
  const router = useRouter();
  const [wrong, setWrong] = useState("");

  async function change(password: string, form: FormData) {
    const current = String(form.get("current") ?? "");
    const send = () => browserApi("/identity/me/password", { method: "POST", body: { current, new: password } });
    setWrong("");
    try {
      await send();
    } catch (failure) {
      if (!(failure instanceof ApiError && failure.code === "identity.step_up_required")) throw failure;
      await browserApi("/identity/auth/step-up", { method: "POST", body: { password: current } });
      await send();
    }
    router.replace("/sign-in?changed=password");
    router.refresh();
  }

  return (
    <Card component="section" aria-labelledby="password-title">
      <CardContent sx={{ p: { xs: 2.5, md: 4 } }}>
        <Typography id="password-title" variant="h3" component="h2" sx={{ mb: 2.5 }}>
          {t("password.title")}
        </Typography>
        <Box sx={{ maxInlineSize: 480 }}>
          <NewPasswordForm
            label={t("password.title")}
            submit={t("password.save")}
            onSubmit={change}
            describe={(failure) => {
              if (failure instanceof ApiError && (failure.code === "identity.password_wrong" || failure.status === 401)) {
                setWrong(t("password.current.wrong"));
                return t("password.current.wrong");
              }
              return undefined;
            }}
          >
            <PasswordField name="current" label={t("password.current")} autoComplete="current-password" required error={wrong || undefined} onChange={() => setWrong("")} />
          </NewPasswordForm>
        </Box>
      </CardContent>
    </Card>
  );
}

/** The open sessions of the person, each with its device, when it started and was last used, and a way to close it. */
function Sessions({ sessions }: { readonly sessions: readonly SessionSummary[] }) {
  const { t, date, relative } = useI18n();
  const toast = useToast();
  const [items, setItems] = useState(sessions);
  const [all, setAll] = useState(false);
  const name = (client: string | null | undefined) => {
    const device = deviceOf(client);
    if (!device) return t("sessions.unknown");
    return device.system ? t("sessions.device", { browser: device.browser, system: device.system }) : device.browser;
  };

  async function close(id: string) {
    try {
      await browserApi(`/identity/me/sessions/${id}`, { method: "DELETE" });
      setItems((list) => list.filter((s) => s.id !== id));
      toast(t("sessions.closed"), "success");
    } catch {
      toast(t("error.generic"), "danger");
    }
  }

  return (
    <Card component="section" aria-labelledby="sessions-title">
      <CardContent sx={{ p: { xs: 2.5, md: 4 } }}>
        <Typography id="sessions-title" variant="h3" component="h2">
          {t("sessions.title")}
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5, mb: 2 }}>
          {t("sessions.lead")}
        </Typography>
        {items.length === 0 ? (
          <EmptyState compact icon="devices" message={t("sessions.empty")} />
        ) : (
          <Stack spacing={1}>
            {(all ? items : items.slice(0, SHOWN)).map((session) => (
              <Stack key={session.id} direction={{ xs: "column", sm: "row" }} spacing={2} sx={{ alignItems: { sm: "center" }, p: 1.5, borderRadius: 3, border: 1, borderColor: "divider" }}>
                <Box sx={{ display: "grid", placeItems: "center", inlineSize: 40, blockSize: 40, flex: "none", borderRadius: 2.5, backgroundImage: "var(--k-palette-gradient-soft)", color: "primary.main" }}>
                  <Icon name="devices" size="small" />
                </Box>
                <Box sx={{ flex: 1, minInlineSize: 0 }}>
                  <Typography variant="subtitle2" noWrap>
                    {name(session.client)}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {[session.address, t("sessions.started", { date: date(session.created_at) }), session.last_seen_at ? t("sessions.seen", { date: relative(session.last_seen_at) }) : null]
                      .filter(Boolean)
                      .join(" · ")}
                  </Typography>
                </Box>
                <Button variant="outlined" tone="danger" icon="logout" size="small" onClick={() => void close(session.id)}>
                  {t("sessions.close")}
                </Button>
              </Stack>
            ))}
            {!all && items.length > SHOWN ? (
              <Box>
                <Button variant="text" icon="keyboard_arrow_down" onClick={() => setAll(true)}>
                  {t("sessions.all", { n: items.length })}
                </Button>
              </Box>
            ) : null}
          </Stack>
        )}
      </CardContent>
    </Card>
  );
}

/** `<SecurityPanel sessions={…} />` */
export function SecurityPanel({ sessions, factors }: { readonly sessions: readonly SessionSummary[]; readonly factors: readonly SecondFactor[] }) {
  return (
    <StepUpProvider>
      <Stack spacing={3}>
        <PasswordForm />
        <SecondFactors factors={factors} />
        <Sessions sessions={sessions} />
      </Stack>
    </StepUpProvider>
  );
}
