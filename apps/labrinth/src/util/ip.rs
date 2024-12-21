use std::net::{AddrParseError, IpAddr, Ipv6Addr};

pub fn convert_to_ip_v6(src: &str) -> Result<Ipv6Addr, AddrParseError> {
    let ip_addr: IpAddr = src.parse()?;

    Ok(match ip_addr {
        IpAddr::V4(x) => x.to_ipv6_mapped(),
        IpAddr::V6(x) => x,
    })
}

pub fn strip_ip(ip: Ipv6Addr) -> u64 {
    if let Some(ip) = ip.to_ipv4_mapped() {
        let octets = ip.octets();
        u64::from_be_bytes([
            octets[0], octets[1], octets[2], octets[3], 0, 0, 0, 0,
        ])
    } else {
        let octets = ip.octets();
        u64::from_be_bytes([
            octets[0], octets[1], octets[2], octets[3], octets[4], octets[5],
            octets[6], octets[7],
        ])
    }
}
