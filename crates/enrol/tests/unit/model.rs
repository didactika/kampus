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
// @package   kampus-enrol
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The values of enrolment as they are stored and sent.

use kampus_enrol::{ExpiryAction, Method, Status};

#[test]
fn methods_read_back_from_their_codes() {
    for method in [Method::Manual, Method::SelfEnrol, Method::SystemGroup] {
        assert_eq!(method.as_str().parse::<Method>().unwrap(), method);
    }
    assert_eq!(
        "payment".parse::<Method>().unwrap_err().code(),
        "enrol.method_unknown"
    );
    assert_eq!(serde_json::to_value(Method::SelfEnrol).unwrap(), "self");
}

#[test]
fn statuses_and_expiry_actions_read_back() {
    for status in [Status::Active, Status::Suspended] {
        assert_eq!(Status::read(status.as_str()), status);
    }
    for action in [
        ExpiryAction::Suspend,
        ExpiryAction::Remove,
        ExpiryAction::KeepWithoutRole,
    ] {
        assert_eq!(ExpiryAction::read(action.as_str()), action);
    }
}
