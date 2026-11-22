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

/** Activating an account from its link, an invitation or a sign-up, by choosing the password. */

import { useState } from "react";

import { Loading } from "@kampus/ui";

import { browserApi } from "../../api/browser";
import { ApiError } from "../../api/problem";
import { useI18n } from "../../i18n/I18nProvider";
import { NewPasswordForm } from "../session/NewPasswordForm";
import { useLinkToken } from "./link-token";
import { BackToSignIn, Outcome, StepHeading } from "./Outcome";

/** The link that activates the account: an invitation of the campus, or the confirmation of a sign-up. */
export type ActivationKind = "invitation" | "sign_up";

/** The route of the API that completes each kind of link. */
const ROUTES: Readonly<Record<ActivationKind, string>> = {
  invitation: "/identity/auth/invitations/accept",
  sign_up: "/identity/auth/confirm",
};

/** `<Activation kind="invitation" />` */
export function Activation({ kind }: { readonly kind: ActivationKind }) {
  const { t } = useI18n();
  const token = useLinkToken();
  const [done, setDone] = useState(false);

  if (token === undefined) return <Loading label={t("loading")} />;
  if (!token) {
    return (
      <Outcome title={t("activate.title")} severity="error">
        {t("link.invalid")}
      </Outcome>
    );
  }
  if (done) {
    return (
      <Outcome title={t("activate.title")} severity="success">
        {t("activate.done")}
      </Outcome>
    );
  }
  return (
    <>
      <StepHeading title={t("activate.title")} lead={t("activate.lead")} />
      <NewPasswordForm
        wide
        label={t("activate.title")}
        submit={t("password.store")}
        describe={(failure) => (failure instanceof ApiError && failure.status < 500 ? t("link.invalid") : undefined)}
        onSubmit={async (password) => {
          await browserApi(ROUTES[kind], { method: "POST", body: { token, password } });
          setDone(true);
        }}
      />
      <BackToSignIn />
    </>
  );
}
