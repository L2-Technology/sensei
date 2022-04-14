use std::{path::{PathBuf, Path}, io::{Cursor}, fs::{self}, ops::Deref, sync::Arc, collections::HashMap, net::SocketAddr};

use bitcoin::{BlockHash, Txid, Network, blockdata::constants::genesis_block, hashes::hex::FromHex};
use lightning::{util::{persist::KVStorePersister, ser::{Writeable, Readable}}, chain::{keysinterface::{Sign, KeysInterface}, channelmonitor::ChannelMonitor}, routing::{network_graph::NetworkGraph, scoring::{ProbabilisticScorer, ProbabilisticScoringParameters}}};
use lightning_persister::FilesystemPersister;
use bitcoin::secp256k1::key::PublicKey;
use lightning::util::ser::ReadableArgs;

use crate::node;


pub trait KVStoreReader {
  fn read(&self, key: &str) -> std::io::Result<Option<Vec<u8>>>;
  fn list(&self, key: &str) -> std::io::Result<Vec<String>>;
}

pub struct FileStore {
  filesystem_persister: FilesystemPersister
}

impl KVStorePersister for FileStore {
  fn persist<W: Writeable>(&self, key: &str, object: &W) -> std::io::Result<()> {
    self.filesystem_persister.persist(key, object)
  }
}

impl KVStoreReader for FileStore {
  fn read(&self, key: &str) -> std::io::Result<Option<Vec<u8>>> {
    let full_path = format!("{}/{}", self.filesystem_persister.get_data_dir(), key);
    let path = PathBuf::from(full_path);
    match fs::read(path) {
      Ok(contents) => Ok(Some(contents)),
      Err(_) => Ok(None)
    }
  }

  fn list(&self, key: &str) -> std::io::Result<Vec<String>> {
    let path = format!("{}/{}", self.filesystem_persister.get_data_dir(), key);
		if !Path::new(&PathBuf::from(&path)).exists() {
			return Ok(Vec::new());
		}
		let mut res = Vec::new();
		for file_option in fs::read_dir(path).unwrap() {
			let file = file_option.unwrap();
			let owned_file_name = file.file_name();
      if let Some(filename) = owned_file_name.to_str() {
        res.push(filename.to_string())
      }
    }
    Ok(res)
  }
}

impl FileStore {
  pub fn new(root: String) -> Self {
    Self {
      filesystem_persister: FilesystemPersister::new(root)
    }
  }
}

pub enum AnyKVStore {
  File(FileStore)
}

impl KVStorePersister for AnyKVStore {
  fn persist<W: Writeable>(&self, key: &str, object: &W) -> std::io::Result<()> {
    match self {
      AnyKVStore::File(store) => store.persist(key, object)
    }
  }
}

impl KVStoreReader for AnyKVStore {
  fn read(&self, key: &str) -> std::io::Result<Option<Vec<u8>>> {
    match self {
      AnyKVStore::File(store) => store.read(key)
    }
  }

  fn list(&self, key: &str) -> std::io::Result<Vec<String>> {
    match self {
      AnyKVStore::File(store) => store.list(key)
    }
  }
}

pub struct SenseiPersister {
  store: AnyKVStore,
  network: Network,
}

impl SenseiPersister {
  pub fn new(store: AnyKVStore, network: Network) -> Self {
    Self { store, network }
  }

  pub fn read_channel_manager(&self) -> std::io::Result<Option<Vec<u8>>> {
    self.store.read("manager")
  }

  pub fn read_network_graph(&self) -> NetworkGraph {
    if let Ok(Some(contents)) = self.store.read("network_graph") {
      let mut cursor = Cursor::new(contents);
      if let Ok(graph) = NetworkGraph::read(&mut cursor) {
        return graph;
      }
    }

    let genesis_hash = genesis_block(self.network).header.block_hash();    
    NetworkGraph::new(genesis_hash)
  }

  pub fn read_scorer(&self, network_graph: Arc<NetworkGraph>) -> ProbabilisticScorer<Arc<NetworkGraph>> {
    let params = ProbabilisticScoringParameters::default();
    if let Ok(Some(contents)) = self.store.read("scorer") {
      let mut cursor = Cursor::new(contents);
      if let Ok(scorer) = ProbabilisticScorer::read(&mut cursor, (params, Arc::clone(&network_graph))) {
        return scorer;
      }
    }
    ProbabilisticScorer::new(params, network_graph)
  }

  pub fn persist_scorer(&self, scorer: &ProbabilisticScorer<Arc<NetworkGraph>>) -> std::io::Result<()> {
    self.store.persist("scorer", scorer)  
  }

  fn get_raw_channel_peer_data(&self) -> String {
    if let Ok(Some(contents)) = self.store.read("channel_peer_data") {
      if let Ok(channel_peer_data) = String::read(&mut Cursor::new(contents)) {
        return channel_peer_data
      }
    }

    String::new()
  }

  pub fn persist_channel_peer(&self, peer_info: &str) -> std::io::Result<()> {
    let mut peer_data = self.get_raw_channel_peer_data();
    peer_data.push_str( peer_info);
    peer_data.push_str("\n");
    self.store.persist("channel_peer_data", &peer_data)
  }

  pub fn read_channel_peer_data(&self) -> Result<HashMap<PublicKey, SocketAddr>, std::io::Error> {
    let mut peer_data = HashMap::new();
    let raw_peer_data = self.get_raw_channel_peer_data();
    for line in raw_peer_data.lines() {
        match node::parse_peer_info(line.to_string()) {
            Ok((pubkey, socket_addr)) => {
                peer_data.insert(pubkey, socket_addr);
            }
            Err(e) => return Err(e),
        }
    }
    Ok(peer_data)
  }

  /// Read `ChannelMonitor`s from disk.
	pub fn read_channelmonitors<Signer: Sign, K: Deref> (
		&self, keys_manager: K
	) -> Result<Vec<(BlockHash, ChannelMonitor<Signer>)>, std::io::Error>
		where K::Target: KeysInterface<Signer=Signer> + Sized,
	{
		let filenames = self.store.list("monitors").unwrap();

		let mut res = Vec::new();
		for filename in filenames {
			if !filename.is_ascii() || filename.len() < 65 {
				return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					"Invalid ChannelMonitor file name",
				));
			}
			if filename.ends_with(".tmp") {
				// If we were in the middle of committing an new update and crashed, it should be
				// safe to ignore the update - we should never have returned to the caller and
				// irrevocably committed to the new state in any way.
				continue;
			}

			let txid = Txid::from_hex(filename.split_at(64).0);
			if txid.is_err() {
				return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					"Invalid tx ID in filename",
				));
			}

			let index: Result<u16, _> = filename.split_at(65).1.parse();
			if index.is_err() {
				return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					"Invalid tx index in filename",
				));
			}

      let monitor_path = format!("monitors/{}", filename);
      let contents = self.store.read(&monitor_path)?.unwrap();
			let mut buffer = Cursor::new(&contents);
			match <(BlockHash, ChannelMonitor<Signer>)>::read(&mut buffer, &*keys_manager) {
				Ok((blockhash, channel_monitor)) => {
					if channel_monitor.get_funding_txo().0.txid != txid.unwrap() || channel_monitor.get_funding_txo().0.index != index.unwrap() {
						return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "ChannelMonitor was stored in the wrong file"));
					}
					res.push((blockhash, channel_monitor));
				}
				Err(e) => return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					format!("Failed to deserialize ChannelMonitor: {}", e),
				))
			}
		}
		Ok(res)
	}
}

impl KVStorePersister for SenseiPersister {
  fn persist<W: Writeable>(&self, key: &str, object: &W) -> std::io::Result<()> {
    self.store.persist(key, object)
  }
}