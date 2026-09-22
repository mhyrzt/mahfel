//! Resolve the caller's source address for rate limiting.
//!
//! Behind a reverse proxy the socket peer is the proxy, so using it directly
//! would put every caller in one bucket. `X-Forwarded-For` is only honoured when
//! the peer is the configured trusted proxy; otherwise any client could forge
//! the header and obtain a fresh budget.

use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::request::Parts,
};
use std::net::{IpAddr, SocketAddr};
use tokio::net::lookup_host;

/// Resolves the address an attempt is attributed to.
#[derive(Clone, Debug, Default)]
pub struct SourceAddressResolver {
    /// Socket addresses whose `X-Forwarded-For` may be trusted.
    trusted_proxies: Vec<SocketAddr>,
}

impl SourceAddressResolver {
    /// Build from the configured trusted proxy, resolving a hostname once at
    /// startup.
    ///
    /// # Errors
    /// Returns the resolver's reason when a configured proxy cannot be
    /// resolved, so a typo fails loudly at boot rather than silently disabling
    /// header trust.
    pub async fn new(trusted_proxy: Option<&str>) -> Result<Self, String> {
        let Some(configured) = trusted_proxy else {
            return Ok(Self::default());
        };
        // Accept either a bare address or `host:port`; a bare host needs a port
        // to be comparable to a socket peer, so try both forms.
        let candidates = [configured.to_owned(), format!("{configured}:0")];
        for candidate in candidates {
            if let Ok(resolved) = lookup_host(candidate.as_str()).await {
                let addresses: Vec<SocketAddr> = resolved.collect();
                if !addresses.is_empty() {
                    return Ok(Self {
                        trusted_proxies: addresses,
                    });
                }
            }
        }
        Err(format!("trusted_proxy `{configured}` did not resolve"))
    }

    /// The address to attribute an attempt to.
    ///
    /// Prefers the forwarded address only when the peer is a trusted proxy and
    /// the header carries a parseable IP; falls back to the peer.
    #[must_use]
    pub fn resolve(&self, peer: IpAddr, forwarded_for: Option<&str>) -> Option<String> {
        let peer_socket = SocketAddr::new(peer, 0);
        let trusted = self
            .trusted_proxies
            .iter()
            .any(|proxy| proxy.ip() == peer_socket.ip());
        if trusted && let Some(ip) = forwarded_for.and_then(first_forwarded_ip) {
            return Some(ip.to_string());
        }
        Some(peer.to_string())
    }
}

/// The originating client from an `X-Forwarded-For` list.
///
/// The list is appended to by each hop, so the first entry is the original
/// client. An unparseable entry stops the chain rather than being skipped,
/// because a forged list must not be walked past.
fn first_forwarded_ip(header: &str) -> Option<IpAddr> {
    let first = header.split(',').next()?.trim();
    first.parse().ok()
}

/// The socket peer, when the server was started with connection info.
///
/// Absent in-process, where tests call the router directly rather than through a
/// listener, so it is optional rather than a rejection.
pub struct PeerAddress(pub Option<IpAddr>);

impl<S> FromRequestParts<S> for PeerAddress
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(address)| address.ip()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolver_trusting(proxy: &str) -> SourceAddressResolver {
        SourceAddressResolver {
            trusted_proxies: vec![SocketAddr::new(proxy.parse().expect("ip"), 0)],
        }
    }

    #[test]
    fn an_untrusted_peer_cannot_forge_the_header() {
        let resolver = resolver_trusting("10.0.0.5");
        let peer: IpAddr = "203.0.113.9".parse().expect("ip");
        assert_eq!(
            resolver.resolve(peer, Some("1.2.3.4")),
            Some("203.0.113.9".to_owned()),
            "a client that is not the proxy must be attributed to its own address"
        );
    }

    #[test]
    fn a_trusted_peer_contributes_the_forwarded_address() {
        let resolver = resolver_trusting("10.0.0.5");
        let peer: IpAddr = "10.0.0.5".parse().expect("ip");
        assert_eq!(
            resolver.resolve(peer, Some("1.2.3.4")),
            Some("1.2.3.4".to_owned())
        );
    }

    #[test]
    fn the_first_entry_of_a_chain_is_used() {
        let resolver = resolver_trusting("10.0.0.5");
        let peer: IpAddr = "10.0.0.5".parse().expect("ip");
        assert_eq!(
            resolver.resolve(peer, Some("1.2.3.4, 10.0.0.5, 10.0.0.6")),
            Some("1.2.3.4".to_owned())
        );
    }

    #[test]
    fn a_forged_leading_entry_does_not_walk_the_chain() {
        let resolver = resolver_trusting("10.0.0.5");
        let peer: IpAddr = "10.0.0.5".parse().expect("ip");
        assert_eq!(
            resolver.resolve(peer, Some("not-an-ip, 10.0.0.5")),
            Some("10.0.0.5".to_owned()),
            "an unparseable entry must stop the chain, not be skipped"
        );
    }

    #[test]
    fn no_trusted_proxy_means_the_peer_is_always_used() {
        let resolver = SourceAddressResolver::default();
        let peer: IpAddr = "203.0.113.9".parse().expect("ip");
        assert_eq!(
            resolver.resolve(peer, Some("1.2.3.4")),
            Some("203.0.113.9".to_owned())
        );
    }

    #[test]
    fn a_missing_forwarded_header_falls_back_to_the_peer() {
        let resolver = resolver_trusting("10.0.0.5");
        let peer: IpAddr = "10.0.0.5".parse().expect("ip");
        assert_eq!(resolver.resolve(peer, None), Some("10.0.0.5".to_owned()));
    }
}
