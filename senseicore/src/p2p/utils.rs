use bitcoin::secp256k1::PublicKey;
use lightning::ln::msgs::NetAddress;
use socks::{TargetAddr, ToTargetAddr};
use std::net::{IpAddr, SocketAddr};

use crate::hex_utils;

pub fn net_address_to_socket_addr(net_address: NetAddress) -> Option<SocketAddr> {
    match net_address {
        NetAddress::IPv4 { addr, port } => Some(SocketAddr::new(IpAddr::from(addr), port)),
        NetAddress::IPv6 { addr, port } => Some(SocketAddr::new(IpAddr::from(addr), port)),
        NetAddress::OnionV2(_) => None,
        NetAddress::OnionV3 { .. } => None,
        NetAddress::Hostname { .. } => None,
    }
}

pub async fn parse_peer_addr(peer_addr_str: &str) -> Result<NetAddress, std::io::Error> {
    let peer_addr = peer_addr_str.to_target_addr();

    if peer_addr.is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ERROR: couldn't parse host:port into a target address",
        ));
    }

    let addr = peer_addr.unwrap();

    match addr {
        TargetAddr::Ip(socket_addr) => {
            let listen_addr = public_ip::addr()
                .await
                .unwrap_or_else(|| [127, 0, 0, 1].into());

            let connect_address = match listen_addr == socket_addr.ip() {
                true => format!("127.0.0.1:{}", socket_addr.port()).parse().unwrap(),
                false => socket_addr,
            };

            match connect_address {
                SocketAddr::V4(v4) => Ok(NetAddress::IPv4 {
                    addr: v4.ip().octets(),
                    port: v4.port(),
                }),
                SocketAddr::V6(v6) => Ok(NetAddress::IPv6 {
                    addr: v6.ip().octets(),
                    port: v6.port(),
                }),
            }
        }
        TargetAddr::Domain(_host, _port) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ERROR: we don't support tor addresses yet",
        )),
    }
}

pub async fn parse_peer_info(
    peer_pubkey_and_ip_addr: String,
) -> Result<(PublicKey, NetAddress), std::io::Error> {
    let mut pubkey_and_addr = peer_pubkey_and_ip_addr.split('@');
    let pubkey = pubkey_and_addr.next();
    let peer_addr_str = pubkey_and_addr.next();
    if pubkey.is_none() || peer_addr_str.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ERROR: incorrectly formatted peer info. Should be formatted as: `pubkey@host:port`",
        ));
    }

    let pubkey = parse_pubkey(pubkey.unwrap())?;
    let connect_address = parse_peer_addr(peer_addr_str.unwrap()).await?;

    Ok((pubkey, connect_address))
}

pub fn parse_pubkey(pubkey: &str) -> Result<PublicKey, std::io::Error> {
    let pubkey = hex_utils::to_compressed_pubkey(pubkey);
    if pubkey.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ERROR: unable to parse given pubkey for node",
        ));
    }
    Ok(pubkey.unwrap())
}
