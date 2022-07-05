use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::node::{NetworkGraph, RoutingPeerManager};
use crate::persist::SenseiPersister;

use super::router::AnyScorer;

const PING_TIMER: u64 = 10;
/// Prune the network graph of stale entries hourly.
const NETWORK_PRUNE_TIMER: u64 = 60 * 60;
const SCORER_PERSIST_TIMER: u64 = 30;
const FIRST_NETWORK_PRUNE_TIMER: u64 = 60;

pub struct BackgroundProcessor {
    peer_manager: Arc<RoutingPeerManager>,
    scorer: Arc<Mutex<AnyScorer>>,
    network_graph: Arc<NetworkGraph>,
    persister: Arc<SenseiPersister>,
}

impl BackgroundProcessor {
    pub fn new(
        peer_manager: Arc<RoutingPeerManager>,
        scorer: Arc<Mutex<AnyScorer>>,
        network_graph: Arc<NetworkGraph>,
        persister: Arc<SenseiPersister>,
    ) -> Self {
        Self {
            peer_manager,
            scorer,
            network_graph,
            persister,
        }
    }

    pub async fn process(&self) {
        let mut last_prune_call = Instant::now();
        let mut last_scorer_persist_call = Instant::now();
        let mut last_ping_call = Instant::now();
        let mut have_pruned = false;
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;
            if last_ping_call.elapsed().as_secs() > PING_TIMER {
                self.peer_manager.process_events();
                self.peer_manager.timer_tick_occurred();
                last_ping_call = Instant::now();
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
        }
    }
}
