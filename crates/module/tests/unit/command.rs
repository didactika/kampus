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
// @package   kampus-module
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The commands of components: usage and arguments.

use kampus_common::ErrorKind;
use kampus_module::{Argument, Command, CommandContext, CommandFuture};

/// A command that does nothing.
fn nothing<'c>(_context: &'c CommandContext, _arguments: &'c [String]) -> CommandFuture<'c> {
    Box::pin(async { Ok(Vec::new()) })
}

/// A command with one required and one optional argument.
const GRANT: Command = Command {
    name: "grant",
    summary: "Grant a role",
    arguments: &[
        Argument {
            name: "user",
            summary: "The username",
            required: true,
        },
        Argument {
            name: "until",
            summary: "When it ends",
            required: false,
        },
    ],
    run: nothing,
};

#[test]
fn the_usage_names_required_and_optional_arguments() {
    assert_eq!(GRANT.usage("access"), "kampus-server access grant <user> [until]");
}

#[test]
fn the_number_of_arguments_is_checked() {
    let args = |n: usize| (0..n).map(|i| i.to_string()).collect::<Vec<_>>();
    assert!(GRANT.check("access", &args(1)).is_ok());
    assert!(GRANT.check("access", &args(2)).is_ok());
    for wrong in [0, 3] {
        let error = GRANT.check("access", &args(wrong)).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Validation);
        assert!(
            error
                .params()
                .iter()
                .any(|(k, v)| k == "usage" && v.contains("<user>"))
        );
    }
}
