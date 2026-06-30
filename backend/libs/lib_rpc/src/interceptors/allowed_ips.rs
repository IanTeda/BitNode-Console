//! Allowed-IPs interceptor for gRPC request access control.

use std::net::IpAddr;

/// Interceptor that restricts incoming RPC connections to an allowlist of IP networks.
///
/// The remote address is read from the request extensions, where Tonic places the
/// client [`std::net::SocketAddr`] for each accepted TCP connection. If the address
/// is absent or not covered by any listed network, the request is rejected with
/// [`tonic::Code::PermissionDenied`].
///
/// IPv4-mapped IPv6 addresses (e.g. `::ffff:127.0.0.1`) are normalised to plain
/// IPv4 before comparison, so listing `127.0.0.1/32` covers both forms.
///
/// An empty `allowed` list rejects every request.
#[derive(Clone)]
pub struct AllowedIpsInterceptor {
    allowed: Vec<ipnet::IpNet>,
}

impl AllowedIpsInterceptor {
    /// Create a new [`AllowedIpsInterceptor`] that permits addresses within the given networks.
    pub fn new(allowed: Vec<ipnet::IpNet>) -> Self {
        Self { allowed }
    }
}

impl tonic::service::Interceptor for AllowedIpsInterceptor {
    #[tracing::instrument(name = "allowed_ips.call", skip_all, level = "debug")]
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let ip = request.remote_addr().map(|addr| canonical_ip(addr.ip())).ok_or_else(|| {
            crate::Error::PermissionDenied("remote address unavailable".to_string())
        })?;

        if self.allowed.iter().any(|net| net.contains(&ip)) {
            tracing::debug!(%ip, "allowed IP accepted");
            Ok(request)
        } else {
            tracing::warn!(%ip, "request from disallowed IP rejected");
            Err(crate::Error::PermissionDenied(format!("{ip} is not in the allowed list")).into())
        }
    }
}

/// Normalise IPv4-mapped IPv6 addresses (e.g. `::ffff:127.0.0.1`) to plain IPv4.
#[tracing::instrument(level = "trace")]
fn canonical_ip(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(IpAddr::V6(v6)),
        v4 => v4,
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use super::AllowedIpsInterceptor;
    use tonic::service::Interceptor as _;

    fn net(cidr: &str) -> ipnet::IpNet {
        cidr.parse().expect("valid CIDR string")
    }

    fn ipv4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(a, b, c, d))
    }

    fn interceptor_allowing(nets: Vec<ipnet::IpNet>) -> AllowedIpsInterceptor {
        AllowedIpsInterceptor::new(nets)
    }

    fn request_from(addr: SocketAddr) -> tonic::Request<()> {
        let mut request = tonic::Request::new(());
        request.extensions_mut().insert(tonic::transport::server::TcpConnectInfo {
            local_addr: None,
            remote_addr: Some(addr),
        });
        request
    }

    fn empty_request() -> tonic::Request<()> {
        tonic::Request::new(())
    }

    // --- missing remote address ---

    #[test]
    fn rejects_request_with_no_remote_address() {
        assert!(interceptor_allowing(vec![net("127.0.0.1/32")]).call(empty_request()).is_err());
    }

    #[test]
    fn missing_address_returns_permission_denied() {
        let err = interceptor_allowing(vec![net("127.0.0.1/32")])
            .call(empty_request())
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    // --- disallowed IP ---

    #[test]
    fn rejects_ip_not_in_allowed_list() {
        let addr = SocketAddr::new(ipv4(192, 168, 1, 99), 1234);
        let err = interceptor_allowing(vec![net("10.0.0.1/32")])
            .call(request_from(addr))
            .unwrap_err();
        assert_eq!(err.code(), tonic::Code::PermissionDenied);
    }

    #[test]
    fn empty_allowed_list_rejects_all_requests() {
        let addr = SocketAddr::new(ipv4(127, 0, 0, 1), 1234);
        assert!(interceptor_allowing(vec![]).call(request_from(addr)).is_err());
    }

    #[test]
    fn rejects_ip_outside_cidr_subnet() {
        let addr = SocketAddr::new(ipv4(10, 0, 1, 1), 1234);
        assert!(interceptor_allowing(vec![net("10.0.0.0/24")]).call(request_from(addr)).is_err());
    }

    // --- allowed IP ---

    #[test]
    fn accepts_ip_in_allowed_list() {
        let addr = SocketAddr::new(ipv4(127, 0, 0, 1), 1234);
        assert!(interceptor_allowing(vec![net("127.0.0.1/32")]).call(request_from(addr)).is_ok());
    }

    #[test]
    fn accepts_second_ip_in_multi_entry_list() {
        let allowed = vec![net("10.0.0.1/32"), net("10.0.0.2/32")];
        let addr = SocketAddr::new(ipv4(10, 0, 0, 2), 5678);
        assert!(interceptor_allowing(allowed).call(request_from(addr)).is_ok());
    }

    #[test]
    fn accepts_ip_within_cidr_subnet() {
        let addr = SocketAddr::new(ipv4(10, 0, 0, 42), 1234);
        assert!(interceptor_allowing(vec![net("10.0.0.0/24")]).call(request_from(addr)).is_ok());
    }

    // --- IPv4-mapped IPv6 normalisation ---

    #[test]
    fn accepts_ipv4_mapped_ipv6_when_ipv4_is_allowed() {
        let mapped = IpAddr::V6(Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped());
        let addr = SocketAddr::new(mapped, 1234);
        assert!(interceptor_allowing(vec![net("127.0.0.1/32")]).call(request_from(addr)).is_ok());
    }

    // --- Clone ---

    #[test]
    fn interceptor_is_clone() {
        let _copy = interceptor_allowing(vec![]).clone();
    }
}
