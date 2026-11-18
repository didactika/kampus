# This file is part of Kampus - Knowledge Academic Modular Platform for Universal Study.
#
# Copyright (c) 2026 Didactika. All rights reserved.
# Proprietary and confidential: see the LICENSE file at the root of the repository.
# SPDX-License-Identifier: LicenseRef-Didactika-Proprietary
#
# @package   tools/dev
# @author    hector-ae21
# @since     0.1.0

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
