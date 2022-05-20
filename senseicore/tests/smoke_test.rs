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
    use senseicore::services::{PaginationRequest, PaymentsFilter};
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
            balance == 100_000_000
        };

        assert!(wait_until(has_balance, 15000, 250).await);
    }

    async fn create_node(
        admin_service: &AdminService,
        username: &str,
        passphrase: &str,
        start: bool,
    ) -> Arc<LightningNode> {
        let node_pubkey = match admin_service
            .call(AdminRequest::CreateNode {
                username: String::from(username),
                passphrase: String::from(passphrase),
                alias: String::from(username),
                start,
            })
            .await
            .unwrap()
        {
            AdminResponse::CreateNode {
                id,
                listen_addr,
                listen_port,
                pubkey,
                macaroon,
            } => Some(pubkey),
            _ => None,
        }
        .unwrap();

        let directory = admin_service.node_directory.lock().await;
        let handle = directory.get(&node_pubkey).unwrap();
        handle.node.clone()
    }

    async fn create_root_node(
        admin_service: &AdminService,
        username: &str,
        passphrase: &str,
        start: bool,
    ) -> Arc<LightningNode> {
        match admin_service
            .call(AdminRequest::CreateAdmin {
                username: String::from(username),
                alias: String::from(username),
                passphrase: String::from(passphrase),
                start,
            })
            .await
            .unwrap()
        {
            AdminResponse::CreateAdmin {
                pubkey,
                macaroon,
                id,
                token,
                role,
            } => {
                let directory = admin_service.node_directory.lock().await;
                let handle = directory.get(&pubkey).unwrap();
                Some(handle.node.clone())
            }
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

    async fn wait_until_async<F: Future<Output = bool>, G: Fn() -> F>(
        func: G,
        timeout_ms: u64,
        interval_ms: u64,
    ) -> bool {
        let mut current_ms = 0;
        while current_ms < timeout_ms {
            if func().await {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            current_ms += interval_ms;
        }

        return false;
    }

    async fn open_channel(
        bitcoind: &BitcoinD,
        from: Arc<LightningNode>,
        to: Arc<LightningNode>,
        amt_sat: u64,
    ) {
        let miner_address = bitcoind.client.get_new_address(None, None).unwrap();
        let node_connection_string = format!(
            "{}@{}:{}",
            to.get_pubkey(),
            to.listen_addresses.first().unwrap(),
            to.listen_port
        );

        let mut event_receiver = from.event_sender.subscribe();

        from.call(NodeRequest::OpenChannel {
            node_connection_string: node_connection_string,
            amt_satoshis: amt_sat,
            public: true,
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

        assert!(wait_until(Box::new(has_usable_channel), 15000, 250).await);
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
        let (event_sender, mut event_receiver): (
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

        AdminService::new(
            &sensei_dir,
            config.clone(),
            database,
            chain_manager,
            event_sender,
        )
        .await
    }

    fn run_test<F>(test: fn(BitcoinD, AdminService) -> F) -> F::Output
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
                let sensei_dir = String::from("./.sensei-tests");
                let bitcoind = setup_bitcoind();
                let admin_service =
                    setup_sensei(&sensei_dir, &bitcoind, persistence_runtime_handle).await;
                test(bitcoind, admin_service).await
            })
    }

    async fn smoke_test(bitcoind: BitcoinD, admin_service: AdminService) {
        let alice = create_root_node(&admin_service, "alice", "alice", true).await;
        let bob = create_node(&admin_service, "bob", "bob", true).await;
        let charlie = create_node(&admin_service, "charlie", "charlie", true).await;
        fund_node(&bitcoind, alice.clone()).await;
        fund_node(&bitcoind, bob.clone()).await;
        open_channel(&bitcoind, alice.clone(), bob.clone(), 1_000_000).await;
        open_channel(&bitcoind, bob.clone(), charlie.clone(), 1_000_000).await;

        let num_invoices = 25;

        let invoices = batch_create_invoices(charlie.clone(), 10, num_invoices).await;

        future::try_join_all(
            invoices
                .into_iter()
                .map(|invoice| pay_invoice(alice.clone(), invoice))
                .map(tokio::spawn),
        )
        .await
        .unwrap();

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
            pagination.total == num_invoices as u64
        };

        assert!(wait_until(has_payments, 60000, 500).await);
    }

    #[test]
    fn run_smoke_test() {
        run_test(smoke_test)
    }
}
