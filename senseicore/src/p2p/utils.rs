use bitcoin::secp256k1::PublicKey;
use std::net::{SocketAddr, ToSocketAddrs};

use crate::hex_utils;

pub async fn parse_peer_addr(peer_addr_str: &str) -> Result<SocketAddr, std::io::Error> {
    let peer_addr = peer_addr_str.to_socket_addrs().map(|mut r| r.next());

    if peer_addr.is_err() || peer_addr.as_ref().unwrap().is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "ERROR: couldn't parse host:port into a socket address",
        ));
    }

    let addr = peer_addr.unwrap().unwrap();

    let listen_addr = public_ip::addr()
        .await
        .unwrap_or_else(|| [127, 0, 0, 1].into());

    let connect_address = match listen_addr == addr.ip() {
        true => format!("127.0.0.1:{}", addr.port()).parse().unwrap(),
        false => addr,
    };

    Ok(connect_address)
}

pub async fn parse_peer_info(
    peer_pubkey_and_ip_addr: String,
) -> Result<(PublicKey, SocketAddr), std::io::Error> {
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
