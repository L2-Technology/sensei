use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::hashes::{Hash, HashEngine};
use bitcoin::secp256k1;
use bitcoin::secp256k1::ecdh::SharedSecret;
use bitcoin::secp256k1::key::{PublicKey, SecretKey};
use bitcoin::secp256k1::Secp256k1;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::node::PeerManager;

use super::noise::{decrypt_with_ad, hkdf, BidirectionalNoiseState};

const NOISE_CK: [u8; 32] = [
    0x26, 0x40, 0xf5, 0x2e, 0xeb, 0xcd, 0x9e, 0x88, 0x29, 0x58, 0x95, 0x1c, 0x79, 0x42, 0x50, 0xee,
    0xdb, 0x28, 0x00, 0x2c, 0x05, 0xd7, 0xdc, 0x2e, 0xa0, 0xf1, 0x95, 0x40, 0x60, 0x42, 0xca, 0xf1,
];
const NOISE_H: [u8; 32] = [
    0xd1, 0xfb, 0xf6, 0xde, 0xe4, 0xf6, 0x86, 0xf1, 0x32, 0xfd, 0x70, 0x2c, 0x4a, 0xbf, 0x8f, 0xba,
    0x4b, 0xb4, 0x20, 0xd8, 0x9d, 0x2a, 0x04, 0x8a, 0x3c, 0x4f, 0x4c, 0x09, 0x2e, 0x37, 0xb6, 0x76,
];

async fn get_peer_manager_for_stream(
    stream: &TcpStream,
    port: u16,
    managers: Arc<Mutex<HashMap<u16, Vec<(SecretKey, Arc<PeerManager>)>>>>,
) -> Option<Arc<PeerManager>> {
    let mut buf: [u8; 50] = [0; 50];
    let mut bytes_read = 0;
    let mut attempts = 0;
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    while bytes_read != 50 && attempts < 10 {
        bytes_read = stream.peek(&mut buf).await.unwrap();
        attempts += 1;

        if bytes_read != 50 {
            interval.tick().await;
        }
    }

    if bytes_read == 50 {
        if let Ok(their_pub) = PublicKey::from_slice(&buf[1..34]) {
            let secp_ctx = Secp256k1::signing_only();

            let mut peer_managers = managers.lock().await;
            let peer_managers = peer_managers.entry(port).or_insert(vec![]);

            for (secret_key, peer_manager) in peer_managers {
                let mut sha = Sha256::engine();
                sha.input(&NOISE_H);
                let our_node_id = PublicKey::from_secret_key(&secp_ctx, &secret_key);
                sha.input(&our_node_id.serialize()[..]);
                let h = Sha256::from_engine(sha).into_inner();

                let mut state = BidirectionalNoiseState { h, ck: NOISE_CK };

                let mut sha = Sha256::engine();
                sha.input(&state.h);
                sha.input(&their_pub.serialize()[..]);
                state.h = Sha256::from_engine(sha).into_inner();

                let ss = SharedSecret::new(&their_pub, &secret_key);
                let temp_k = hkdf(&mut state, ss);

                let mut dec = [0; 0];

                if decrypt_with_ad(&mut dec, 0, &temp_k, &state.h, &buf[34..]) {
                    return Some(peer_manager.clone());
                }
            }
        }
    }

    None
}

// Need to be able to get a mutable list of Keys + PeerManagers for a given port when
// registering a new PeerManager
//
// When a new connection comes in we need a list of (K,PM) to find the PM to give
// the new connection to
pub struct SenseiConnectionManager {
    managers: Arc<Mutex<HashMap<u16, Vec<(SecretKey, Arc<PeerManager>)>>>>,
}

impl SenseiConnectionManager {
    pub fn new() -> Self {
        Self {
            managers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register(&self, port: u16, secret_key: SecretKey, manager: Arc<PeerManager>) {
        let mut managers = self.managers.lock().await;
        let peer_managers = managers.entry(port).or_insert(vec![]);
        peer_managers.push((secret_key, manager));

        if peer_managers.len() == 1 {
            self.setup_listener(port);
        }
    }

    pub fn setup_listener(&self, port: u16) {
        let listener_managers = self.managers.clone();

        tokio::spawn(async move {
            let addr = format!("0.0.0.0:{}", port);
            let listener = tokio::net::TcpListener::bind(addr).await.expect(
                "Failed to bind to listen port - is something else already listening on it?",
            );

            loop {
                let tcp_stream = listener.accept().await.unwrap().0;
                // if stop_listen_ref.load(Ordering::Acquire) {
                //     return;
                // }
                if let Some(peer_manager) =
                    get_peer_manager_for_stream(&tcp_stream, port, listener_managers.clone()).await
                {
                    tokio::spawn(async move {
                        lightning_net_tokio::setup_inbound(
                            peer_manager,
                            tcp_stream.into_std().unwrap(),
                        )
                        .await;
                    });
                }
            }
        });
    }
}
