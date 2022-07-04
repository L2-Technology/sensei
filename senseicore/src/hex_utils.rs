// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use bitcoin::secp256k1::PublicKey;
use std::fmt::Write;

#[allow(dead_code)]
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

#[inline]
pub fn hex_str(value: &[u8]) -> String {
    let mut res = String::with_capacity(64);
    for v in value {
        let _ = write!(res, "{:02x}", v);
    }
    res
}

#[allow(dead_code)]
pub fn sanitize_string(bytes: &[u8]) -> String {
    let mut ret = String::with_capacity(bytes.len());
    // We should really support some sane subset of UTF-8 here, but limiting to printable ASCII
    // instead makes this trivial.
    for b in bytes {
        if *b >= 0x20 && *b <= 0x7e {
            ret.push(*b as char);
        }
    }
    ret
}

#[allow(dead_code)]
pub fn to_compressed_pubkey(hex: &str) -> Option<PublicKey> {
    if hex.len() != 33 * 2 {
        return None;
    }
    let data = match to_vec(&hex[0..33 * 2]) {
        Some(bytes) => bytes,
        None => return None,
    };
    match PublicKey::from_slice(&data) {
        Ok(pk) => Some(pk),
        Err(_) => None,
    }
}
