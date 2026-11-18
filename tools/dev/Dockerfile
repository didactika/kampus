# ------------------------------------------------------------------------------
#
#     _  __    _    __  __ ____  _   _ ____
#    | |/ /   / \  |  \/  |  _ \| | | / ___|
#    | ' /   / _ \ | |\/| | |_) | | | \___ \
#    | . \  / ___ \| |  | |  __/| |_| |___) |
#    |_|\_\/_/   \_\_|  |_|_|    \___/|____/
#
#    Knowledge Academic Modular Platform for Universal Study
#
# ------------------------------------------------------------------------------
#
# This file is part of Kampus - https://github.com/didactika/kampus
#
# Kampus is free software: you can redistribute it and/or modify it under the terms of the
# GNU Affero General Public License as published by the Free Software Foundation, either
# version 3 of the License, or (at your option) any later version.
#
# Kampus is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without
# even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along with Kampus.
# If not, see <https://www.gnu.org/licenses/>.
#
# @package   tools/dev
# @copyright 2026 Didactika
# @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
# @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
# @since     0.1.0
#
# SPDX-License-Identifier: AGPL-3.0-or-later
# ------------------------------------------------------------------------------

FROM rust:1.100.0-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends git ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

ARG CARGO_DENY=0.21.0
ARG CARGO_AUDIT=0.22.2

RUN curl -fsSL "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY}/cargo-deny-${CARGO_DENY}-x86_64-unknown-linux-musl.tar.gz" \
    | tar -xz --strip-components=1 -C /usr/local/bin "cargo-deny-${CARGO_DENY}-x86_64-unknown-linux-musl/cargo-deny" \
    && curl -fsSL "https://github.com/rustsec/rustsec/releases/download/cargo-audit%2Fv${CARGO_AUDIT}/cargo-audit-x86_64-unknown-linux-musl-v${CARGO_AUDIT}.tgz" \
    | tar -xz --strip-components=1 -C /usr/local/bin "cargo-audit-x86_64-unknown-linux-musl-v${CARGO_AUDIT}/cargo-audit"

RUN rustup component add rustfmt clippy

WORKDIR /work
