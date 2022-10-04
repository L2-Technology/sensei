#[cfg(test)]
mod test {
    use bitcoin::{Address, Amount, Network};
    use bitcoincore_rpc::RpcApi;
    use bitcoind::BitcoinD;
    use entity::sea_orm::{ConnectOptions, Database};
    use futures::{future, Future};
    use migration::{Migrator, MigratorTrait};
    use senseicore::events::SenseiEvent;
    use senseicore::node::{HTLCStatus, LightningNode};
    use senseicore::services::node::{Channel, OpenChannelRequest};
    use senseicore::services::{PaginationRequest, PaymentsFilter};
    use serial_test::serial;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::{str::FromStr, sync::Arc, time::Duration};
    use tokio::runtime::{Builder, Handle};
    use tokio::sync::broadcast;

    use senseicore::{
        chain::{bitcoind_client::BitcoindClient, manager::SenseiChainManager},
        config::SenseiConfig,
        database::SenseiDatabase,
        services::node::{NodeRequest, NodeResponse},
    };

    use senseicore::services::admin::{AdminRequest, AdminResponse, AdminService};

    async fn fund_node(bitcoind: &BitcoinD, node: Arc<LightningNode>) {
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();
        let fund_address = match node.call(NodeRequest::GetUnusedAddress {}).await.unwrap() {
            NodeResponse::GetUnusedAddress { address } => Some(address),
            _ => None,
        }
        .unwrap();

        let fund_address = Address::from_str(&fund_address).unwrap();

        let _res = bitcoind
            .client
            .send_to_address(
                &fund_address,
                Amount::from_btc(1.0).unwrap(),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        bitcoind
            .client
            .generate_to_address(1, &miner_address)
            .unwrap();

        let closed_node = node.clone();
        let has_balance = move || {
            let wallet = closed_node.wallet.lock().unwrap();
            let balance = wallet.get_balance().unwrap();
            balance.get_total() == 100_000_000
        };

        assert!(wait_until(has_balance, 15000, 250).await);
    }

    async fn create_node(
        admin_service: &AdminService,
        username: &str,
        passphrase: &str,
        start: bool,
        entropy: Option<String>,
        cross_node_entropy: Option<String>,
    ) -> (Arc<LightningNode>, String, String) {
        let (node_pubkey, entropy, cross_node_entropy) = match admin_service
            .call(AdminRequest::CreateNode {
                username: String::from(username),
                passphrase: String::from(passphrase),
                alias: String::from(username),
                start,
                entropy,
                cross_node_entropy,
            })
            .await
            .unwrap()
        {
            AdminResponse::CreateNode {
                id: _,
                listen_addr: _,
                listen_port: _,
                pubkey,
                macaroon: _,
                entropy,
                cross_node_entropy,
            } => Some((pubkey, entropy, cross_node_entropy)),
            _ => None,
        }
        .unwrap();

        let directory = admin_service.node_directory.lock().await;
        let handle = directory.get(&node_pubkey).unwrap();
        let node = handle.as_ref().unwrap().node.clone();
        (node, entropy, cross_node_entropy)
    }

    async fn create_admin_account(
        admin_service: &AdminService,
        username: &str,
        passphrase: &str,
    ) -> String {
        match admin_service
            .call(AdminRequest::CreateAdmin {
                username: String::from(username),
                passphrase: String::from(passphrase),
            })
            .await
            .unwrap()
        {
            AdminResponse::CreateAdmin { token } => Some(token),
            _ => None,
        }
        .unwrap()
    }

    async fn wait_for_event<F: Fn(SenseiEvent) -> bool>(
        event_receiver: &mut broadcast::Receiver<SenseiEvent>,
        filter: F,
        timeout_ms: u64,
        interval_ms: u64,
    ) -> Option<SenseiEvent> {
        let mut current_ms = 0;
        while current_ms < timeout_ms {
            while let Ok(event) = event_receiver.try_recv() {
                if filter(event.clone()) {
                    return Some(event);
                }
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            current_ms += interval_ms;
        }
        return None;
    }

    async fn wait_until<F: Fn() -> bool>(func: F, timeout_ms: u64, interval_ms: u64) -> bool {
        let mut current_ms = 0;
        while current_ms < timeout_ms {
            if func() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            current_ms += interval_ms;
        }

        return false;
    }

    async fn close_channel(
        bitcoind: &BitcoinD,
        from: Arc<LightningNode>,
        to: Arc<LightningNode>,
        channel: Channel,
        force: bool,
    ) {
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();
        let mut event_receiver = from.event_sender.subscribe();

        from.call(NodeRequest::CloseChannel {
            channel_id: channel.channel_id.clone(),
            force,
        })
        .await
        .unwrap();

        let close_node_id = from.id.clone();
        let filter = move |event| {
            if let SenseiEvent::TransactionBroadcast { node_id, .. } = event {
                if *node_id == close_node_id {
                    return true;
                }
            }
            return false;
        };

        let event = wait_for_event(&mut event_receiver, filter.clone(), 15000, 250).await;
        assert!(event.is_some());

        let blocks_needed = match force {
            true => 144,
            false => 20,
        };

        let mut from_event_receiver = from.event_sender.subscribe();
        let mut to_event_receiver = to.event_sender.subscribe();

        bitcoind
            .client
            .generate_to_address(blocks_needed, &miner_address)
            .unwrap();

        let from_channel = from.clone();
        let to_channel = to.clone();
        let channel_is_gone = move || {
            let from_channels = from_channel
                .list_channels(PaginationRequest {
                    page: 0,
                    take: 10,
                    query: Some(channel.channel_id.clone()),
                })
                .unwrap()
                .0;

            let to_channels = to_channel
                .list_channels(PaginationRequest {
                    page: 0,
                    take: 10,
                    query: Some(channel.channel_id.clone()),
                })
                .unwrap()
                .0;
            from_channels.len() == 0 && to_channels.len() == 0
        };

        assert!(wait_until(Box::new(channel_is_gone), 15000, 250).await);

        let from_sweep_node_id = from.id.clone();
        let from_sweep_filter = move |event| {
            if let SenseiEvent::TransactionBroadcast { node_id, .. } = event {
                if *node_id == from_sweep_node_id {
                    return true;
                }
            }
            return false;
        };

        let to_sweep_node_id = to.id.clone();
        let to_sweep_filter = move |event| {
            if let SenseiEvent::TransactionBroadcast { node_id, .. } = event {
                if *node_id == to_sweep_node_id {
                    return true;
                }
            }
            return false;
        };

        // wait for sweep txs
        let event = wait_for_event(&mut from_event_receiver, from_sweep_filter, 120000, 250).await;
        assert!(event.is_some());

        let event = wait_for_event(&mut to_event_receiver, to_sweep_filter, 120000, 250).await;
        assert!(event.is_some());

        bitcoind
            .client
            .generate_to_address(20, &miner_address)
            .unwrap();
    }

    async fn get_onchain_balance_sats(node: Arc<LightningNode>) -> u64 {
        match node.call(NodeRequest::GetBalance {}).await.unwrap() {
            NodeResponse::GetBalance {
                onchain_balance_sats,
                ..
            } => Some(onchain_balance_sats),
            _ => None,
        }
        .unwrap()
    }

    async fn get_channel_balance_sats(node: Arc<LightningNode>) -> u64 {
        match node.call(NodeRequest::GetBalance {}).await.unwrap() {
            NodeResponse::GetBalance {
                channel_balance_msats,
                ..
            } => Some(channel_balance_msats / 1000),
            _ => None,
        }
        .unwrap()
    }

    async fn open_channels(
        bitcoind: &BitcoinD,
        from: Arc<LightningNode>,
        to: Vec<Arc<LightningNode>>,
        amt_sat: u64,
    ) -> Vec<(Channel, Arc<LightningNode>)> {
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();

        let channel_requests = to
            .iter()
            .map(|to| OpenChannelRequest {
                counterparty_pubkey: to.get_pubkey(),
                counterparty_host_port: Some(format!(
                    "{}:{}",
                    to.listen_addresses.first().unwrap(),
                    to.listen_port
                )),
                amount_sats: amt_sat,
                public: true,
                scid_alias: None,
                custom_id: None,
                push_amount_msats: None,
                forwarding_fee_proportional_millionths: None,
                forwarding_fee_base_msat: None,
                cltv_expiry_delta: None,
                max_dust_htlc_exposure_msat: None,
                force_close_avoidance_max_fee_satoshis: None,
            })
            .collect::<Vec<OpenChannelRequest>>();

        let mut event_receiver = from.event_sender.subscribe();

        from.call(NodeRequest::OpenChannels {
            requests: channel_requests,
        })
        .await
        .unwrap();

        let from_node_id = from.id.clone();
        let filter = move |event| {
            if let SenseiEvent::TransactionBroadcast { node_id, .. } = event {
                if *node_id == from_node_id {
                    return true;
                }
            }
            return false;
        };

        let event = wait_for_event(&mut event_receiver, filter, 15000, 250).await;
        assert!(event.is_some());

        // TODO: looks like I can just remove this?
        let _funding_txid = match event.unwrap() {
            SenseiEvent::TransactionBroadcast { txid, .. } => Some(txid),
            _ => None,
        }
        .unwrap();

        bitcoind
            .client
            .generate_to_address(10, &miner_address)
            .unwrap();

        let usable_from = from.clone();
        let expected_channels = to.len();
        let has_usable_channels = move || {
            let usable_channels = usable_from
                .list_channels(PaginationRequest {
                    page: 0,
                    take: 5,
                    query: None,
                })
                .unwrap()
                .0
                .into_iter()
                .filter(|channel| channel.is_usable)
                .collect::<Vec<Channel>>();

            usable_channels.len() >= expected_channels
        };
        assert!(wait_until(Box::new(has_usable_channels), 15000, 250).await);

        for to_node in &to {
            let usable_to = to_node.clone();

            let has_usable_channels = move || {
                let usable_channels = usable_to
                    .list_channels(PaginationRequest {
                        page: 0,
                        take: 5,
                        query: None,
                    })
                    .unwrap()
                    .0
                    .into_iter()
                    .filter(|channel| channel.is_usable)
                    .collect::<Vec<Channel>>();

                usable_channels.len() == 1
            };
            assert!(wait_until(Box::new(has_usable_channels), 15000, 250).await);
        }

        from.list_channels(PaginationRequest {
            page: 0,
            take: to.len() as u32,
            query: None,
        })
        .unwrap()
        .0
        .into_iter()
        .map(|channel| {
            let counterparty = to
                .iter()
                .find(|to| to.node_info().unwrap().node_pubkey == channel.counterparty_pubkey)
                .unwrap();
            (channel, counterparty.clone())
        })
        .collect()
    }

    async fn open_channel(
        bitcoind: &BitcoinD,
        from: Arc<LightningNode>,
        to: Arc<LightningNode>,
        amt_sat: u64,
    ) -> Channel {
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();
        let mut event_receiver = from.event_sender.subscribe();

        from.call(NodeRequest::OpenChannels {
            requests: vec![OpenChannelRequest {
                counterparty_pubkey: to.get_pubkey(),
                counterparty_host_port: Some(format!(
                    "{}:{}",
                    to.listen_addresses.first().unwrap(),
                    to.listen_port
                )),
                amount_sats: amt_sat,
                public: true,
                scid_alias: None,
                custom_id: None,
                push_amount_msats: None,
                forwarding_fee_proportional_millionths: None,
                forwarding_fee_base_msat: None,
                cltv_expiry_delta: None,
                max_dust_htlc_exposure_msat: None,
                force_close_avoidance_max_fee_satoshis: None,
            }],
        })
        .await
        .unwrap();

        let from_node_id = from.id.clone();
        let filter = move |event| {
            if let SenseiEvent::TransactionBroadcast { node_id, .. } = event {
                if *node_id == from_node_id {
                    return true;
                }
            }
            return false;
        };

        let event = wait_for_event(&mut event_receiver, filter, 15000, 250).await;
        assert!(event.is_some());

        let funding_txid = match event.unwrap() {
            SenseiEvent::TransactionBroadcast { txid, .. } => Some(txid),
            _ => None,
        }
        .unwrap();

        bitcoind
            .client
            .generate_to_address(10, &miner_address)
            .unwrap();

        let has_usable_channel = move || {
            let channels = to
                .list_channels(PaginationRequest {
                    page: 0,
                    take: 5,
                    query: None,
                })
                .unwrap()
                .0;
            channels.len() > 0 && channels[0].is_usable
        };

        assert!(wait_until(Box::new(has_usable_channel), 30000, 250).await);

        from.list_channels(PaginationRequest {
            page: 0,
            take: 1,
            query: Some(funding_txid.to_string()),
        })
        .unwrap()
        .0[0]
            .clone()
    }

    async fn create_phantom_invoice(
        node: Arc<LightningNode>,
        cluster: Vec<Arc<LightningNode>>,
        amt_sat: u64,
    ) -> String {
        let mut phantom_route_hints_hex = vec![];
        for cluster_node in cluster.iter() {
            let hint = match cluster_node
                .call(NodeRequest::GetPhantomRouteHints {})
                .await
                .unwrap()
            {
                NodeResponse::GetPhantomRouteHints {
                    phantom_route_hints_hex,
                } => Some(phantom_route_hints_hex),
                _ => None,
            }
            .unwrap();
            phantom_route_hints_hex.push(hint);
        }

        match node
            .call(NodeRequest::GetPhantomInvoice {
                amt_msat: amt_sat * 1000,
                description: String::from("test"),
                phantom_route_hints_hex,
            })
            .await
            .unwrap()
        {
            NodeResponse::GetPhantomInvoice { invoice } => Some(invoice),
            _ => None,
        }
        .unwrap()
    }

    async fn create_invoice(node: Arc<LightningNode>, amt_sat: u64) -> String {
        match node
            .call(NodeRequest::GetInvoice {
                amt_msat: amt_sat * 1000,
                description: String::from("test"),
            })
            .await
            .unwrap()
        {
            NodeResponse::GetInvoice { invoice } => Some(invoice),
            _ => None,
        }
        .unwrap()
    }

    async fn batch_create_invoices(
        node: Arc<LightningNode>,
        amt_sat: u64,
        num_invoices: usize,
    ) -> Vec<String> {
        let mut i = 0;
        let mut invoices: Vec<String> = vec![];
        while i < num_invoices {
            let raw_invoice = create_invoice(node.clone(), amt_sat).await;
            invoices.push(raw_invoice);
            i += 1;
        }
        invoices
    }

    async fn pay_invoice(node: Arc<LightningNode>, invoice: String) {
        node.call(NodeRequest::SendPayment { invoice })
            .await
            .unwrap();
    }

    fn within_range(actual: f64, expected: f64, pct_err: f64) -> bool {
        (actual - expected).abs() < expected * pct_err
    }

    fn setup_bitcoind() -> BitcoinD {
        let bitcoind = bitcoind::BitcoinD::new(bitcoind::downloaded_exe_path().unwrap()).unwrap();
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();
        bitcoind
            .client
            .generate_to_address(110, &miner_address)
            .unwrap();
        bitcoind
    }

    fn setup_test_environment(bitcoind: &BitcoinD, sensei_dir: &str) -> SenseiConfig {
        cleanup_test_environment(sensei_dir);
        std::fs::create_dir_all(format!("{}/{}", sensei_dir, Network::Regtest))
            .expect("failed to create data directory");
        let sqlite_path = format!("{}/{}/{}", sensei_dir, Network::Regtest, "sensei.sqlite");

        let mut config = SenseiConfig::default();
        config.network = Network::Regtest;
        config.bitcoind_rpc_host = bitcoind.params.rpc_socket.ip().to_string();
        config.bitcoind_rpc_port = bitcoind.params.rpc_socket.port();
        config.bitcoind_rpc_username = String::from("__cookie__");
        let cookie = std::fs::read_to_string(bitcoind.params.cookie_file.clone()).unwrap();
        let cookie_parts = cookie.split(':').collect::<Vec<&str>>();
        config.bitcoind_rpc_password = cookie_parts.last().unwrap().to_string();
        config.database_url = format!("sqlite://{}?mode=rwc", sqlite_path);
        config
    }

    fn cleanup_test_environment(sensei_dir: &str) {
        std::fs::remove_dir_all(&sensei_dir).unwrap_or_default();
    }

    async fn setup_sensei(
        sensei_dir: &str,
        bitcoind: &BitcoinD,
        persistence_handle: Handle,
    ) -> AdminService {
        let (event_sender, _event_receiver): (
            broadcast::Sender<SenseiEvent>,
            broadcast::Receiver<SenseiEvent>,
        ) = broadcast::channel(256);
        let config = setup_test_environment(&bitcoind, sensei_dir);

        let mut db_connection_options = ConnectOptions::new(config.database_url.clone());
        db_connection_options
            .max_connections(100)
            .min_connections(10)
            .connect_timeout(Duration::new(30, 0));
        let db_connection = Database::connect(db_connection_options).await.unwrap();
        Migrator::up(&db_connection, None)
            .await
            .expect("failed to run migrations");

        let database = SenseiDatabase::new(db_connection, persistence_handle);
        database.mark_all_nodes_stopped().await.unwrap();

        let bitcoind_client = Arc::new(
            BitcoindClient::new(
                config.bitcoind_rpc_host.clone(),
                config.bitcoind_rpc_port,
                config.bitcoind_rpc_username.clone(),
                config.bitcoind_rpc_password.clone(),
                tokio::runtime::Handle::current(),
            )
            .await
            .expect("invalid bitcoind rpc config"),
        );

        let chain_manager = Arc::new(
            SenseiChainManager::new(
                config.clone(),
                bitcoind_client.clone(),
                bitcoind_client.clone(),
                bitcoind_client,
            )
            .await
            .unwrap(),
        );

        let stop_signal = Arc::new(AtomicBool::new(false));

        AdminService::new(
            &sensei_dir,
            config.clone(),
            database,
            chain_manager,
            event_sender,
            tokio::runtime::Handle::current(),
            stop_signal,
        )
        .await
    }

    fn run_test<F>(name: &str, test: fn(BitcoinD, AdminService) -> F) -> F::Output
    where
        F: Future,
    {
        let persistence_runtime = Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("persistence")
            .enable_all()
            .build()
            .unwrap();

        let persistence_runtime_handle = persistence_runtime.handle().clone();

        Builder::new_multi_thread()
            .worker_threads(10)
            .thread_name("sensei")
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                let sensei_dir = format!("./.sensei-tests/{}", name);
                let bitcoind = setup_bitcoind();
                let admin_service =
                    setup_sensei(&sensei_dir, &bitcoind, persistence_runtime_handle).await;
                let output = test(bitcoind, admin_service.clone()).await;
                admin_service.stop_signal.store(true, Ordering::Relaxed);
                admin_service.stop().await.unwrap();
                output
            })
    }

    async fn phantom_payment_test(bitcoind: BitcoinD, admin_service: AdminService) {
        let _admin_token = create_admin_account(&admin_service, "admin", "admin").await;
        let (alice, _alice_entropy, alice_cross_node_entropy) =
            create_node(&admin_service, "alice", "alice", true, None, None).await;
        let (bob, ..) = create_node(
            &admin_service,
            "bob",
            "bob",
            true,
            None,
            Some(alice_cross_node_entropy),
        )
        .await;

        let cluster = vec![alice.clone(), bob.clone()];

        let (charlie, ..) =
            create_node(&admin_service, "charlie", "charlie", true, None, None).await;

        fund_node(&bitcoind, charlie.clone()).await;

        let _charlie_bob_channel =
            open_channel(&bitcoind, charlie.clone(), bob.clone(), 1_000_000).await;

        let phantom_invoice = create_phantom_invoice(alice, cluster, 5000).await;

        let bob_start_balance = get_channel_balance_sats(bob.clone()).await;

        // charlie only has a channel to bob but should be able to pay alice's invoice
        pay_invoice(charlie.clone(), phantom_invoice).await;

        let charlie_test = charlie.clone();
        let has_payments = move || {
            let pagination = PaginationRequest {
                page: 0,
                take: 1,
                query: None,
            };
            let filter = PaymentsFilter {
                status: Some(HTLCStatus::Succeeded.to_string()),
                origin: None,
            };
            let (_payments, pagination) = charlie_test
                .database
                .list_payments_sync(charlie_test.id.clone(), pagination, filter)
                .unwrap();
            pagination.total == 1 as u64
        };

        assert!(wait_until(has_payments, 60000, 500).await);

        let bob_balance = get_channel_balance_sats(bob.clone()).await;
        assert!(bob_balance > bob_start_balance);
    }

    async fn smoke_test(bitcoind: BitcoinD, admin_service: AdminService) {
        let _admin_token = create_admin_account(&admin_service, "admin", "admin").await;
        let (alice, ..) = create_node(&admin_service, "alice", "alice", true, None, None).await;
        let (bob, ..) = create_node(&admin_service, "bob", "bob", true, None, None).await;
        let (charlie, ..) =
            create_node(&admin_service, "charlie", "charlie", true, None, None).await;
        fund_node(&bitcoind, alice.clone()).await;
        fund_node(&bitcoind, bob.clone()).await;
        let alice_bob_channel =
            open_channel(&bitcoind, alice.clone(), bob.clone(), 1_000_000).await;
        let bob_charlie_channel =
            open_channel(&bitcoind, bob.clone(), charlie.clone(), 1_000_000).await;

        let num_invoices = 25;
        let invoice_amt = 3500;
        let invoices = batch_create_invoices(charlie.clone(), invoice_amt, num_invoices).await;

        future::try_join_all(
            invoices
                .into_iter()
                .map(|invoice| pay_invoice(alice.clone(), invoice))
                .map(tokio::spawn),
        )
        .await
        .unwrap();

        let alice_test = alice.clone();
        let has_payments = move || {
            let pagination = PaginationRequest {
                page: 0,
                take: 1,
                query: None,
            };
            let filter = PaymentsFilter {
                status: Some(HTLCStatus::Succeeded.to_string()),
                origin: None,
            };
            let (_payments, pagination) = alice_test
                .database
                .list_payments_sync(alice_test.id.clone(), pagination, filter)
                .unwrap();
            pagination.total == num_invoices as u64
        };

        assert!(wait_until(has_payments, 60000, 500).await);

        close_channel(
            &bitcoind,
            alice.clone(),
            bob.clone(),
            alice_bob_channel,
            false,
        )
        .await;
        close_channel(
            &bitcoind,
            bob.clone(),
            charlie.clone(),
            bob_charlie_channel,
            true,
        )
        .await;

        let alice_balance = get_onchain_balance_sats(alice.clone()).await;
        let bob_balance = get_onchain_balance_sats(bob.clone()).await;
        let charlie_balance = get_onchain_balance_sats(charlie.clone()).await;

        let alice_initial_balance = 100_000_000 as u64;
        let bob_initial_balance = 100_000_000 as u64;
        let charlie_initial_balance = 0 as u64;
        let expected_payment_amount = num_invoices as u64 * invoice_amt;
        let routing_fee_sats = 1 as u64;
        let routing_fees = routing_fee_sats * num_invoices as u64;
        let channel_open_tx_fee = 1224 as u64;
        let channel_close_tx_fee = 2348 as u64;
        let channel_force_close_tx_fee = 1538 as u64;
        let channel_sweep_tx_fee = 876 as u64;

        let alice_expected_balance = alice_initial_balance
            - expected_payment_amount
            - routing_fees
            - channel_open_tx_fee
            - channel_close_tx_fee
            - channel_sweep_tx_fee;
        let bob_expected_balance = bob_initial_balance + routing_fees
            - channel_open_tx_fee
            - channel_force_close_tx_fee
            - 2 * channel_sweep_tx_fee;
        let charlie_expected_balance =
            charlie_initial_balance + expected_payment_amount - channel_sweep_tx_fee;

        // fuzzy match here because fee estimation could change
        assert!(within_range(
            alice_balance as f64,
            alice_expected_balance as f64,
            0.01
        ));
        assert!(within_range(
            bob_balance as f64,
            bob_expected_balance as f64,
            0.01
        ));
        assert!(within_range(
            charlie_balance as f64,
            charlie_expected_balance as f64,
            0.01
        ));
    }

    async fn batch_open_channels_test(bitcoind: BitcoinD, admin_service: AdminService) {
        let _admin_token = create_admin_account(&admin_service, "admin", "admin").await;
        let (alice, ..) = create_node(&admin_service, "alice", "alice", true, None, None).await;
        let (bob, ..) = create_node(&admin_service, "bob", "bob", true, None, None).await;
        let (charlie, ..) =
            create_node(&admin_service, "charlie", "charlie", true, None, None).await;
        let (doug, ..) = create_node(&admin_service, "doug", "doug", true, None, None).await;
        fund_node(&bitcoind, alice.clone()).await;

        let alice_channels_with_counterparties = open_channels(
            &bitcoind,
            alice.clone(),
            vec![bob.clone(), charlie.clone(), doug.clone()],
            1_000_000,
        )
        .await;

        let num_invoices = 5;
        let invoice_amt = 3500;
        let mut bob_invoices = batch_create_invoices(bob.clone(), invoice_amt, num_invoices).await;
        let mut charlie_invoices =
            batch_create_invoices(charlie.clone(), invoice_amt, num_invoices).await;
        let mut doug_invoices =
            batch_create_invoices(doug.clone(), invoice_amt, num_invoices).await;
        let mut invoices = vec![];
        invoices.append(&mut bob_invoices);
        invoices.append(&mut charlie_invoices);
        invoices.append(&mut doug_invoices);
        future::try_join_all(
            invoices
                .into_iter()
                .map(|invoice| pay_invoice(alice.clone(), invoice))
                .map(tokio::spawn),
        )
        .await
        .unwrap();

        let alice_test = alice.clone();
        let has_payments = move || {
            let pagination = PaginationRequest {
                page: 0,
                take: 1,
                query: None,
            };
            let filter = PaymentsFilter {
                status: Some(HTLCStatus::Succeeded.to_string()),
                origin: None,
            };
            let (_payments, pagination) = alice_test
                .database
                .list_payments_sync(alice_test.id.clone(), pagination, filter)
                .unwrap();
            pagination.total == (num_invoices * 3) as u64
        };

        assert!(wait_until(has_payments, 60000, 500).await);

        for (channel, counterparty) in alice_channels_with_counterparties {
            close_channel(&bitcoind, alice.clone(), counterparty, channel, false).await
        }
    }

    #[test]
    #[serial]
    fn run_batch_open_channel_test() {
        run_test("batch_open_channels", batch_open_channels_test)
    }

    #[test]
    #[serial]
    fn run_smoke_test() {
        run_test("smoke_test", smoke_test)
    }

    #[test]
    #[serial]
    fn run_phantom_payment_test() {
        run_test("phantom_payment", phantom_payment_test)
    }
}
