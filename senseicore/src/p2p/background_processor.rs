use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::node::{NetworkGraph, RoutingPeerManager};
use crate::persist::SenseiPersister;

use lightning_rapid_gossip_sync::RapidGossipSync;

use super::router::AnyScorer;

const PING_TIMER: u64 = 10;
/// Prune the network graph of stale entries hourly.
const NETWORK_PRUNE_TIMER: u64 = 60 * 60;
const FIRST_NETWORK_PRUNE_TIMER: u64 = 60;
const SCORER_PERSIST_TIMER: u64 = 30;
const RAPID_GOSSIP_SYNC_TIMER: u64 = 60 * 60;
const FIRST_RAPID_GOSSIP_SYNC_TIMER: u64 = 0;

pub struct BackgroundProcessor {
    peer_manager: Option<Arc<RoutingPeerManager>>,
    scorer: Arc<Mutex<AnyScorer>>,
    network_graph: Arc<NetworkGraph>,
    persister: Arc<SenseiPersister>,
    stop_signal: Arc<AtomicBool>,
    rapid_gossip_sync_server_host: Option<String>,
}

impl BackgroundProcessor {
    pub fn new(
        peer_manager: Option<Arc<RoutingPeerManager>>,
        scorer: Arc<Mutex<AnyScorer>>,
        network_graph: Arc<NetworkGraph>,
        persister: Arc<SenseiPersister>,
        stop_signal: Arc<AtomicBool>,
        rapid_gossip_sync_server_host: Option<String>,
    ) -> Self {
        Self {
            peer_manager,
            scorer,
            network_graph,
            persister,
            stop_signal,
            rapid_gossip_sync_server_host,
        }
    }

    pub async fn process(&self) {
        let mut last_prune_call = Instant::now();
        let mut last_scorer_persist_call = Instant::now();
        let mut last_ping_call = Instant::now();
        let mut last_rgs_sync_call = Instant::now();
        let mut have_pruned = false;
        let mut have_rapid_gossip_synced = false;
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;

            if let Some(peer_manager) = &self.peer_manager {
                if last_ping_call.elapsed().as_secs() > PING_TIMER {
                    peer_manager.process_events();
                    peer_manager.timer_tick_occurred();
                    last_ping_call = Instant::now();
                }
            }

            // Note that we want to run a graph prune once not long after startup before
            // falling back to our usual hourly prunes. This avoids short-lived clients never
            // pruning their network graph. We run once 60 seconds after startup before
            // continuing our normal cadence.
            if last_prune_call.elapsed().as_secs()
                > if have_pruned {
                    NETWORK_PRUNE_TIMER
                } else {
                    FIRST_NETWORK_PRUNE_TIMER
                }
            {
                self.network_graph.remove_stale_channels();
                if let Err(e) = self.persister.persist_graph(&self.network_graph) {
                    println!("Error: Failed to persist network graph, check your disk and permissions {}", e);
                }

                last_prune_call = Instant::now();
                have_pruned = true;
            }

            if let Some(rapid_gossip_sync_server_host) = &self.rapid_gossip_sync_server_host {
                if last_rgs_sync_call.elapsed().as_secs()
                    > if have_rapid_gossip_synced {
                        RAPID_GOSSIP_SYNC_TIMER
                    } else {
                        FIRST_RAPID_GOSSIP_SYNC_TIMER
                    }
                {
                    let rapid_sync = RapidGossipSync::new(self.network_graph.clone());
                    let last_rapid_gossip_sync_timestamp = self
                        .network_graph
                        .get_last_rapid_gossip_sync_timestamp()
                        .unwrap_or(0);
                    let rapid_gossip_sync_uri = format!(
                        "{}/snapshot/{}",
                        rapid_gossip_sync_server_host, last_rapid_gossip_sync_timestamp
                    );
                    let update_data = match reqwest::get(&rapid_gossip_sync_uri).await {
                        Ok(response) => {
                            match response.bytes().await {
                                Ok(bytes) => Some(bytes.to_vec()),
                                Err(e) => {
                                    eprintln!("failed to convert rapid gossip sync response to bytes: {:?}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "failed to fetch rapid gossip sync update at {} with error: {:?}",
                                rapid_gossip_sync_uri, e
                            );
                            None
                        }
                    };

                    if let Some(update_data) = update_data {
                        if let Err(e) = rapid_sync.update_network_graph(&update_data[..]) {
                            eprintln!(
                                "failed to update network graph with rapid gossip sync data: {:?}",
                                e
                            );
                        }
                    }

                    last_rgs_sync_call = Instant::now();
                    have_rapid_gossip_synced = true;
                }
            }

            if last_scorer_persist_call.elapsed().as_secs() > SCORER_PERSIST_TIMER {
                let scorer = self.scorer.lock().unwrap();
                if let AnyScorer::Local(scorer) = scorer.deref() {
                    if self.persister.persist_scorer(scorer).is_err() {
                        // Persistence errors here are non-fatal as channels will be re-scored as payments
                        // fail, but they may indicate a disk error which could be fatal elsewhere.
                        eprintln!(
                            "Warning: Failed to persist scorer, check your disk and permissions"
                        );
                    }
                }
                last_scorer_persist_call = Instant::now();
            }

            if self.stop_signal.load(Ordering::Acquire) {
                break;
            }
        }

        let scorer = self.scorer.lock().unwrap();
        if let AnyScorer::Local(scorer) = scorer.deref() {
            if self.persister.persist_scorer(scorer).is_err() {
                // Persistence errors here are non-fatal as channels will be re-scored as payments
                // fail, but they may indicate a disk error which could be fatal elsewhere.
                eprintln!("Warning: Failed to persist scorer, check your disk and permissions");
            }
        }

        if let Err(_e) = self.persister.persist_graph(&self.network_graph) {
            eprintln!("Warning: Failed to persist graph, check your disk and permissions");
        }
    }
}
