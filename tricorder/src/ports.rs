use crate::{
    common_ports::MOST_COMMON_PORTS_100,
    model::{Port, Subdomain},
};
use rayon::prelude::*;
use std::net::{SocketAddr, ToSocketAddrs};
use std::{net::TcpStream, time::Duration};

pub fn scan_ports(mut subdomain: Subdomain) -> Subdomain {
    subdomain.ports = MOST_COMMON_PORTS_100
        .into_par_iter()
        .map(|port| scan_port(&subdomain.domain, *port))
        .filter(|port| port.is_open)
        .collect();

    subdomain
}

fn scan_port(hostname: &str, port: u16) -> Port {
    let timeout = Duration::from_secs(3);
    let socket_addresses: Vec<SocketAddr> = format!("{}:{}", hostname, port)
        .to_socket_addrs()
        .expect("port scanner: Creating socket addresses")
        .collect();

    let is_open = socket_addresses.first().map_or(false, |addr| {
        TcpStream::connect_timeout(addr, timeout).is_ok()
    });

    Port { port, is_open }
}
