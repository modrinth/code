use std::net::{AddrParseError, IpAddr, Ipv6Addr};

use actix_web::HttpRequest;

use crate::env::ENV;

pub fn convert_to_ip_v6(src: &str) -> Result<Ipv6Addr, AddrParseError> {
    let ip_addr: IpAddr = src.parse()?;

    Ok(match ip_addr {
        IpAddr::V4(x) => x.to_ipv6_mapped(),
        IpAddr::V6(x) => x,
    })
}

pub fn client_ip(req: &HttpRequest) -> Option<String> {
    if ENV.CLOUDFLARE_INTEGRATION
        && let Some(header) = req.headers().get("CF-Connecting-IP")
    {
        return header.to_str().ok().map(str::to_owned);
    }

    req.connection_info().peer_addr().map(str::to_owned)
}
