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
// @package   kampus-mail-smtp
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The mail adapter for SMTP: `smtp://host:port` sends in the clear, `smtp://host:port?tls=required` with
//! STARTTLS, and `smtps://host:port` over TLS from the start. The sender comes from the `from` option, and
//! the `username` and `password` options sign in when the server asks for it.

mod store;

use kampus_mail::Mailer;

pub use store::Smtp;

/// The registration of this adapter.
pub const ADAPTER: kampus_mail::Adapter = kampus_mail::Adapter {
    manifest: kampus_common::manifest!(),
    name: "smtp",
    schemes: &["smtp", "smtps"],
    open: |settings| {
        Box::pin(async move { Ok(std::sync::Arc::new(Smtp::open(&settings)?) as std::sync::Arc<dyn Mailer>) })
    },
};
