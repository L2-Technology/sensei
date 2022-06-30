use bitcoin::secp256k1::PublicKey;

use crate::node::{PeerManager, RoutingPeerManager};
use std::{net::SocketAddr, sync::Arc, time::Duration};

pub struct PeerConnector {
    pub routing_peer_manager: Arc<RoutingPeerManager>,
}

impl PeerConnector {
    pub async fn connect_peer(
        &self,
        pubkey: PublicKey,
        peer_addr: SocketAddr,
        peer_manager: Arc<PeerManager>,
    ) -> Result<(), ()> {
        match lightning_net_tokio::connect_outbound(Arc::clone(&peer_manager), pubkey, peer_addr)
            .await
        {
            Some(connection_closed_future) => {
                let mut connection_closed_future = Box::pin(connection_closed_future);
                loop {
                    match futures::poll!(&mut connection_closed_future) {
                        std::task::Poll::Ready(_) => {
                            println!("ERROR: Peer disconnected before we finished the handshake");
                            return Err(());
                        }
                        std::task::Poll::Pending => {}
                    }
                    // Avoid blocking the tokio context by sleeping a bit
                    match peer_manager
                        .get_peer_node_ids()
                        .iter()
                        .find(|id| **id == pubkey)
                    {
                        Some(_) => break,
                        None => tokio::time::sleep(Duration::from_millis(10)).await,
                    }
                }
            }
            None => {
                //println!("ERROR: failed to connect to peer");
                return Err(());
            }
        }
        Ok(())
    }

    pub async fn connect_peer_if_necessary(
        &self,
        pubkey: PublicKey,
        peer_addr: SocketAddr,
        peer_manager: Arc<PeerManager>,
    ) -> Result<(), ()> {
        if !peer_manager.get_peer_node_ids().contains(&pubkey) {
            self.connect_peer(pubkey, peer_addr, peer_manager).await?
        }
        Ok(())
    }
}
