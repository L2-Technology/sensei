use bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::hashes::{Hash, HashEngine};
use bitcoin::secp256k1::ecdh::SharedSecret;
use bitcoin::secp256k1::{Message, Secp256k1, SecretKey, Signature, Signing};

use crate::error::Error;

use super::chachapoly1305rfc::ChaCha20Poly1305RFC;

pub struct BidirectionalNoiseState {
    pub h: [u8; 32],
    pub ck: [u8; 32],
}

macro_rules! hkdf_extract_expand {
    ($salt: expr, $ikm: expr) => {{
        let mut hmac = HmacEngine::<Sha256>::new($salt);
        hmac.input($ikm);
        let prk = Hmac::from_engine(hmac).into_inner();
        let mut hmac = HmacEngine::<Sha256>::new(&prk[..]);
        hmac.input(&[1; 1]);
        let t1 = Hmac::from_engine(hmac).into_inner();
        let mut hmac = HmacEngine::<Sha256>::new(&prk[..]);
        hmac.input(&t1);
        hmac.input(&[2; 1]);
        (t1, Hmac::from_engine(hmac).into_inner(), prk)
    }};
    ($salt: expr, $ikm: expr, 2) => {{
        let (k1, k2, _) = hkdf_extract_expand!($salt, $ikm);
        (k1, k2)
    }};
    ($salt: expr, $ikm: expr, 3) => {{
        let (k1, k2, prk) = hkdf_extract_expand!($salt, $ikm);

        let mut hmac = HmacEngine::<Sha256>::new(&prk[..]);
        hmac.input(&k2);
        hmac.input(&[3; 1]);
        (k1, k2, Hmac::from_engine(hmac).into_inner())
    }};
}

pub fn hkdf_extract_expand_twice(salt: &[u8], ikm: &[u8]) -> ([u8; 32], [u8; 32]) {
    hkdf_extract_expand!(salt, ikm, 2)
}

pub fn hkdf_extract_expand_thrice(salt: &[u8], ikm: &[u8]) -> ([u8; 32], [u8; 32], [u8; 32]) {
    hkdf_extract_expand!(salt, ikm, 3)
}

pub fn hkdf(state: &mut BidirectionalNoiseState, ss: SharedSecret) -> [u8; 32] {
    let (t1, t2) = hkdf_extract_expand_twice(&state.ck, &ss[..]);
    state.ck = t1;
    t2
}

pub fn decrypt_with_ad(
    res: &mut [u8],
    n: u64,
    key: &[u8; 32],
    h: &[u8],
    cyphertext: &[u8],
) -> bool {
    let mut nonce = [0; 12];
    nonce[4..].copy_from_slice(&n.to_le_bytes()[..]);
    let mut chacha = ChaCha20Poly1305RFC::new(key, &nonce, h);
    chacha.decrypt(
        &cyphertext[0..cyphertext.len() - 16],
        res,
        &cyphertext[cyphertext.len() - 16..],
    )
}

#[inline]
pub fn sign<C: Signing>(ctx: &Secp256k1<C>, msg: &Message, sk: &SecretKey) -> Signature {
    #[cfg(feature = "grind_signatures")]
    let sig = ctx.sign_low_r(msg, sk);
    #[cfg(not(feature = "grind_signatures"))]
    let sig = ctx.sign(msg, sk);
    sig
}
