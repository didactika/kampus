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
# @package   kampus
# @copyright 2026 Didactika
# @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
# @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
# @since     0.1.0
#
# SPDX-License-Identifier: AGPL-3.0-or-later
# ------------------------------------------------------------------------------

FROM rust:1.100.0-slim AS build

WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p kampus-server \
    && cp target/release/kampus-server /kampus-server

FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home kampus \
    && mkdir -p /etc/kampus /var/lib/kampus/files \
    && chown kampus /etc/kampus /var/lib/kampus/files

COPY --from=build /kampus-server /usr/local/bin/kampus-server

USER kampus
ENV KAMPUS_SERVER_HTTP_ADDRESS=0.0.0.0:8080 \
    KAMPUS_CONFIG_FILE=/etc/kampus/config.toml
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --retries=6 CMD curl -fsS http://127.0.0.1:8080/health/live || exit 1

ENTRYPOINT ["kampus-server"]
CMD ["serve"]
