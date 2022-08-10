use std::{fmt::Write, time::SystemTime};

pub use sea_orm;

pub mod prelude;

pub mod access_token;
pub mod keychain;
pub mod kv_store;
pub mod macaroon;
pub mod node;
pub mod payment;
pub mod peer;
pub mod peer_address;
pub mod script_pubkey;
pub mod transaction;
pub mod user;
pub mod utxo;

pub mod seaql_migrations;

pub fn seconds_since_epoch() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap()
}

pub fn to_vec(hex: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(hex.len() / 2);

    let mut b = 0;
    for (idx, c) in hex.as_bytes().iter().enumerate() {
        b <<= 4;
        match *c {
            b'A'..=b'F' => b |= c - b'A' + 10,
            b'a'..=b'f' => b |= c - b'a' + 10,
            b'0'..=b'9' => b |= c - b'0',
            _ => return None,
        }
        if (idx & 1) == 1 {
            out.push(b);
            b = 0;
        }
    }

    Some(out)
}

pub fn to_vec_unsafe(hex: &str) -> Vec<u8> {
    to_vec(hex).unwrap()
}

#[inline]
pub fn hex_str(value: &[u8]) -> String {
    let mut res = String::with_capacity(64);
    for v in value {
        let _ = write!(res, "{:02x}", v);
    }
    res
}
