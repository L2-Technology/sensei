use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::Result;
use bitcoin::bech32::u5;
use bitcoin::secp256k1::ecdsa::RecoverableSignature;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::secp256k1::{All, PublicKey, SecretKey};
use bitcoin::Witness;
use bitcoin::{Address, Network, Script, Transaction, TxOut};
use lightning::chain::keysinterface::{
    KeyMaterial, KeysInterface, Recipient, SpendableOutputDescriptor,
};
use lightning::ln::msgs::DecodeError;
use lightning::ln::script::ShutdownScript;
use lightning_signer::persist::DummyPersister;
use lightning_signer::{bitcoin, lightning};
use log::{debug, error, info};
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::{runtime, task};
use vls_protocol_client::{DynSigner, Error, KeysManagerClient, SpendableKeysInterface, Transport};
use vls_protocol_signer::handler::{Handler, RootHandler};
use vls_protocol_signer::vls_protocol::model::PubKey;
use vls_protocol_signer::vls_protocol::msgs::{self, DeBolt, SerBolt};
use vls_protocol_signer::vls_protocol::serde_bolt::WireString;
use vls_proxy::grpc::adapter::{ChannelRequest, ClientId, HsmdService};
use vls_proxy::grpc::incoming::TcpIncoming;

use super::util::create_spending_transaction;
use super::util::Shutter;

// A VLS client with a null transport.
// Actually runs VLS in-process, but still performs the protocol
struct NullTransport {
    handler: RootHandler,
}

impl NullTransport {
    pub fn new(address: Address) -> Self {
        let persister = Arc::new(DummyPersister);
        let allowlist = vec![address.to_string()];
        info!("allowlist {:?}", allowlist);
        let network = Network::Regtest; // FIXME
        let handler = RootHandler::new(network, 0, None, persister, allowlist);
        NullTransport { handler }
    }
}

impl Transport for NullTransport {
    fn node_call(&self, message_ser: Vec<u8>) -> Result<Vec<u8>, Error> {
        let message = msgs::from_vec(message_ser)?;
        debug!("ENTER node_call {:?}", message);
        let result = self.handler.handle(message).map_err(|e| {
            error!("error in handle: {:?}", e);
            Error::TransportError
        })?;
        debug!("REPLY node_call {:?}", result);
        Ok(result.as_vec())
    }

    fn call(&self, dbid: u64, peer_id: PubKey, message_ser: Vec<u8>) -> Result<Vec<u8>, Error> {
        let message = msgs::from_vec(message_ser)?;
        debug!("ENTER call({}) {:?}", dbid, message);
        let handler = self.handler.for_new_client(0, peer_id, dbid);
        let result = handler.handle(message).map_err(|e| {
            error!("error in handle: {:?}", e);
            Error::TransportError
        })?;
        debug!("REPLY call({}) {:?}", dbid, result);
        Ok(result.as_vec())
    }
}

struct KeysManager {
    client: KeysManagerClient,
    sweep_address: Address,
    node_id: PublicKey,
}

impl KeysInterface for KeysManager {
    type Signer = DynSigner;

    fn get_node_secret(&self, recipient: Recipient) -> Result<SecretKey, ()> {
        self.client.get_node_secret(recipient)
    }

    fn get_destination_script(&self) -> Script {
        self.client.get_destination_script()
    }

    fn get_shutdown_scriptpubkey(&self) -> ShutdownScript {
        self.client.get_shutdown_scriptpubkey()
    }

    fn get_channel_signer(&self, inbound: bool, channel_value_satoshis: u64) -> Self::Signer {
        let client = self
            .client
            .get_channel_signer(inbound, channel_value_satoshis);
        DynSigner::new(client)
    }

    fn get_secure_random_bytes(&self) -> [u8; 32] {
        self.client.get_secure_random_bytes()
    }

    fn read_chan_signer(&self, reader: &[u8]) -> Result<Self::Signer, DecodeError> {
        let signer = self.client.read_chan_signer(reader)?;
        Ok(DynSigner::new(signer))
    }

    fn sign_invoice(
        &self,
        hrp_bytes: &[u8],
        invoice_data: &[u5],
        recipient: Recipient,
    ) -> Result<RecoverableSignature, ()> {
        self.client.sign_invoice(hrp_bytes, invoice_data, recipient)
    }

    fn get_inbound_payment_key_material(&self) -> KeyMaterial {
        self.client.get_inbound_payment_key_material()
    }
}

impl SpendableKeysInterface for KeysManager {
    fn spend_spendable_outputs(
        &self,
        descriptors: &[&SpendableOutputDescriptor],
        outputs: Vec<TxOut>,
        change_destination_script: Script,
        feerate_sat_per_1000_weight: u32,
        _secp_ctx: &Secp256k1<All>,
    ) -> Result<Transaction> {
        info!("ENTER spend_spendable_outputs");
        let mut tx = create_spending_transaction(
            descriptors,
            outputs,
            change_destination_script,
            feerate_sat_per_1000_weight,
        )?;
        let witnesses = self.client.sign_onchain_tx(&tx, descriptors);
        assert_eq!(witnesses.len(), tx.input.len());
        for (idx, w) in witnesses.into_iter().enumerate() {
            tx.input[idx].witness = Witness::from_vec(w);
        }
        Ok(tx)
    }

    fn get_sweep_address(&self) -> Address {
        self.sweep_address.clone()
    }

    fn get_node_id(&self) -> PublicKey {
        self.node_id
    }
}

pub(crate) async fn make_null_signer(
    network: Network,
    ldk_data_dir: String,
    sweep_address: Address,
) -> Box<dyn SpendableKeysInterface<Signer = DynSigner>> {
    let node_id_path = format!("{}/node_id", ldk_data_dir);

    if let Ok(_node_id_hex) = fs::read_to_string(node_id_path.clone()) {
        unimplemented!("read from disk {}", node_id_path);
    } else {
        let transport = NullTransport::new(sweep_address.clone());
        let node_id = transport.handler.node.get_id();
        let client = KeysManagerClient::new(Arc::new(transport), network.to_string());
        let keys_manager = KeysManager {
            client,
            sweep_address,
            node_id,
        };
        fs::write(node_id_path, node_id.to_string()).expect("write node_id");
        Box::new(keys_manager)
    }
}

struct GrpcTransport {
    sender: Sender<ChannelRequest>,
    #[allow(unused)]
    node_secret: SecretKey,
    node_id: PublicKey,
    handle: Handle,
}

impl GrpcTransport {
    async fn new(
        network: Network,
        sender: Sender<ChannelRequest>,
        sweep_address: Address,
    ) -> Result<Self, Error> {
        info!("waiting for signer");
        let init = msgs::HsmdInit2 {
            derivation_style: 0,
            network_name: WireString(network.to_string().into_bytes()),
            dev_seed: None,
            dev_allowlist: vec![WireString(sweep_address.to_string().into_bytes())],
        };
        let init_reply_vec = Self::do_call_async(sender.clone(), init.as_vec(), None).await?;
        let init_reply = msgs::HsmdInit2Reply::from_vec(init_reply_vec)?;
        let node_secret = SecretKey::from_slice(&init_reply.node_secret.0).expect("node secret");
        let secp_ctx = Secp256k1::new();
        let node_id = PublicKey::from_secret_key(&secp_ctx, &node_secret);
        let handle = Handle::current();

        info!("signer connected, node ID {}", node_id);
        Ok(Self {
            sender,
            node_secret,
            node_id,
            handle,
        })
    }

    fn node_id(&self) -> PublicKey {
        self.node_id
    }

    fn do_call(
        handle: &Handle,
        sender: Sender<ChannelRequest>,
        message: Vec<u8>,
        client_id: Option<ClientId>,
    ) -> Result<Vec<u8>, Error> {
        let join = handle.spawn_blocking(move || {
            runtime::Handle::current()
                .block_on(Self::do_call_async(sender, message, client_id))
                .unwrap()
        });
        let result =
            task::block_in_place(|| runtime::Handle::current().block_on(join)).expect("join");
        Ok(result)
    }

    async fn do_call_async(
        sender: Sender<ChannelRequest>,
        message: Vec<u8>,
        client_id: Option<ClientId>,
    ) -> Result<Vec<u8>, Error> {
        // Create a one-shot channel to receive the reply
        let (reply_tx, reply_rx) = oneshot::channel();

        // Send a request to the gRPC handler to send to signer
        let request = ChannelRequest {
            client_id,
            message,
            reply_tx,
        };

        // This can fail if gRPC adapter shut down
        sender
            .send(request)
            .await
            .map_err(|_| Error::TransportError)?;
        let reply = reply_rx.await.map_err(|_| Error::TransportError)?;
        Ok(reply.reply)
    }
}

impl Transport for GrpcTransport {
    fn node_call(&self, message: Vec<u8>) -> Result<Vec<u8>, Error> {
        Self::do_call(&self.handle, self.sender.clone(), message, None)
    }

    fn call(&self, dbid: u64, peer_id: PubKey, message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let client_id = Some(ClientId {
            peer_id: peer_id.0,
            dbid,
        });

        Self::do_call(&self.handle, self.sender.clone(), message, client_id)
    }
}

pub(crate) async fn make_grpc_signer(
    shutter: Shutter,
    signer_handle: Handle,
    vls_port: u16,
    network: Network,
    ldk_data_dir: String,
    sweep_address: Address,
) -> Box<dyn SpendableKeysInterface<Signer = DynSigner>> {
    let node_id_path = format!("{}/node_id", ldk_data_dir);
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, vls_port));
    let incoming = TcpIncoming::new(addr, false, None).expect("listen incoming");

    let server = HsmdService::new(shutter.trigger.clone(), shutter.signal.clone());

    let sender = server.sender();

    signer_handle.spawn(server.start(incoming, shutter.signal));

    let transport = signer_handle
        .spawn(GrpcTransport::new(network, sender, sweep_address.clone()))
        .await
        .expect("join")
        .expect("gRPC transport init");
    let node_id = transport.node_id();

    let client = KeysManagerClient::new(Arc::new(transport), network.to_string());
    let keys_manager = KeysManager {
        client,
        sweep_address,
        node_id,
    };
    fs::write(node_id_path, node_id.to_string()).expect("write node_id");

    Box::new(keys_manager)
}
