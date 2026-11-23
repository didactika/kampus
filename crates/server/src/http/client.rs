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
// @package   kampus-server
// @copyright 2026 Didactika
// @author    Hector Arrechea <hectorlazaroarrechea@gmail.com>
// @license   https://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0-or-later
// @since     0.1.0
//
// SPDX-License-Identifier: AGPL-3.0-or-later
// ------------------------------------------------------------------------------

//! The address of the person behind each request. Behind a proxy, such as the web client or a load balancer, the
//! peer of the connection is the proxy, and counting sign-in failures by it would lock everyone out at once. Only a
//! peer in the trusted networks may tell the address with `X-Forwarded-For`, which is read from the right, skipping
//! the trusted proxies; the first address that is not one is the person's. Without trusted networks, the peer is.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use kampus_module::http::ClientAddress;

/// A network: an address and how many leading bits of it count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Network {
    /// The address.
    address: IpAddr,
    /// The length of the prefix.
    prefix: u8,
}

impl Network {
    /// Reads `10.0.0.0/8`, `fd00::/8` or a single address.
    fn parse(text: &str) -> Option<Self> {
        let (address, prefix) = match text.split_once('/') {
            Some((address, prefix)) => (
                address.trim().parse::<IpAddr>().ok()?,
                Some(prefix.trim().parse::<u8>().ok()?),
            ),
            None => (text.trim().parse::<IpAddr>().ok()?, None),
        };
        let bits = if address.is_ipv4() { 32 } else { 128 };
        let prefix = prefix.unwrap_or(bits);
        (prefix <= bits).then_some(Self { address, prefix })
    }

    /// Whether the network holds an address.
    fn contains(self, address: IpAddr) -> bool {
        match (self.address, address) {
            (IpAddr::V4(net), IpAddr::V4(ip)) => {
                let mask = u32::MAX.checked_shl(32 - u32::from(self.prefix)).unwrap_or(0);
                u32::from(net) & mask == u32::from(ip) & mask
            }
            (IpAddr::V6(net), IpAddr::V6(ip)) => {
                let mask = u128::MAX.checked_shl(128 - u32::from(self.prefix)).unwrap_or(0);
                u128::from(net) & mask == u128::from(ip) & mask
            }
            (IpAddr::V6(_), IpAddr::V4(_)) => false,
            (IpAddr::V4(_), IpAddr::V6(ip)) => ip
                .to_ipv4_mapped()
                .is_some_and(|ip| self.contains(IpAddr::V4(ip))),
        }
    }
}

/// The networks of the proxies in front of the server, whose `X-Forwarded-For` is believed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustedProxies(Vec<Network>);

impl TrustedProxies {
    /// Reads a comma-separated list of networks or addresses; `None` when one cannot be read.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        text.split(',')
            .filter(|part| !part.trim().is_empty())
            .map(Network::parse)
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    /// Whether an address is one of the trusted proxies.
    fn trusts(&self, address: IpAddr) -> bool {
        self.0.iter().any(|network| network.contains(address))
    }

    /// The address of the person, from the peer of the connection and the `X-Forwarded-For` it sent.
    #[must_use]
    pub fn client(&self, peer: IpAddr, forwarded_for: Option<&str>) -> IpAddr {
        if !self.trusts(peer) {
            return peer;
        }
        let hops: Vec<IpAddr> = forwarded_for
            .unwrap_or_default()
            .split(',')
            .filter_map(|hop| hop.trim().parse().ok())
            .collect();
        hops.iter()
            .rev()
            .copied()
            .find(|hop| !self.trusts(*hop))
            .or_else(|| hops.first().copied())
            .unwrap_or(peer)
    }
}

/// Puts the address of the person in the extensions of every request.
pub async fn resolve(
    State(trusted): State<Arc<TrustedProxies>>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(peer) = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip())
    {
        let forwarded = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok());
        let client = trusted.client(peer, forwarded);
        request.extensions_mut().insert(ClientAddress(client));
    }
    next.run(request).await
}
