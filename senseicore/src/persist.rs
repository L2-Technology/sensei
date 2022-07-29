use std::{
    fs::{self},
    io::Cursor,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use super::database::SenseiDatabase;
use crate::{disk::FilesystemLogger, node::NetworkGraph};
use bitcoin::{
    blockdata::constants::genesis_block, hashes::hex::FromHex, BlockHash, Network, Txid,
};
use lightning::util::ser::ReadableArgs;
use lightning::{
    chain::{
        channelmonitor::ChannelMonitor,
        keysinterface::{KeysInterface, Sign},
    },
    routing::scoring::{ProbabilisticScorer, ProbabilisticScoringParameters},
    util::{persist::KVStorePersister, ser::Writeable},
};
use lightning_persister::FilesystemPersister;

pub trait KVStoreReader {
    fn read(&self, key: &str) -> std::io::Result<Option<Vec<u8>>>;
    fn list(&self, key: &str) -> std::io::Result<Vec<String>>;
}

pub struct DatabaseStore {
    database: Arc<SenseiDatabase>,
    node_id: String,
}

impl KVStorePersister for DatabaseStore {
    fn persist<W: Writeable>(&self, key: &str, object: &W) -> std::io::Result<()> {
        let _entry =
            self.database
                .set_value_sync(self.node_id.clone(), key.to_string(), object.encode())?;

        Ok(())
    }
}

impl KVStoreReader for DatabaseStore {
    fn read(&self, key: &str) -> std::io::Result<Option<Vec<u8>>> {
        Ok(self
            .database
            .get_value_sync(self.node_id.clone(), key.to_string())?
            .map(|entry| entry.v))
    }

    fn list(&self, key: &str) -> std::io::Result<Vec<String>> {
        self.database
            .list_keys_sync(self.node_id.clone(), key)
            .map(|full_keys| {
                let replace_str = format!("{}/", key);
                full_keys
                    .iter()
                    .map(|full_key| full_key.replace(&replace_str, ""))
                    .collect()
            })
            .map_err(|e| e.into())
    }
}

impl DatabaseStore {
    pub fn new(database: Arc<SenseiDatabase>, node_id: String) -> Self {
        Self { database, node_id }
    }
}

pub struct FileStore {
    filesystem_persister: FilesystemPersister,
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
            Err(_) => Ok(None),
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
            filesystem_persister: FilesystemPersister::new(root),
        }
    }
}

pub enum AnyKVStore {
    File(FileStore),
    Database(DatabaseStore),
}

impl KVStorePersister for AnyKVStore {
    fn persist<W: Writeable>(&self, key: &str, object: &W) -> std::io::Result<()> {
        match self {
            AnyKVStore::File(store) => store.persist(key, object),
            AnyKVStore::Database(store) => store.persist(key, object),
        }
    }
}

impl KVStoreReader for AnyKVStore {
    fn read(&self, key: &str) -> std::io::Result<Option<Vec<u8>>> {
        match self {
            AnyKVStore::File(store) => store.read(key),
            AnyKVStore::Database(store) => store.read(key),
        }
    }

    fn list(&self, key: &str) -> std::io::Result<Vec<String>> {
        match self {
            AnyKVStore::File(store) => store.list(key),
            AnyKVStore::Database(store) => store.list(key),
        }
    }
}

pub struct SenseiPersister {
    store: AnyKVStore,
    network: Network,
    logger: Arc<FilesystemLogger>,
}

impl SenseiPersister {
    pub fn new(store: AnyKVStore, network: Network, logger: Arc<FilesystemLogger>) -> Self {
        Self {
            store,
            network,
            logger,
        }
    }

    pub fn read_channel_manager(&self) -> std::io::Result<Option<Vec<u8>>> {
        self.store.read("manager")
    }

    pub fn read_network_graph(&self) -> NetworkGraph {
        if let Ok(Some(contents)) = self.store.read("graph") {
            let mut cursor = Cursor::new(contents);
            if let Ok(graph) = NetworkGraph::read(&mut cursor, self.logger.clone()) {
                return graph;
            }
        }

        let genesis_hash = genesis_block(self.network).header.block_hash();
        NetworkGraph::new(genesis_hash, self.logger.clone())
    }

    pub fn read_scorer(
        &self,
        network_graph: Arc<NetworkGraph>,
    ) -> ProbabilisticScorer<Arc<NetworkGraph>, Arc<FilesystemLogger>> {
        let params = ProbabilisticScoringParameters::default();
        if let Ok(Some(contents)) = self.store.read("scorer") {
            let mut cursor = Cursor::new(contents);
            if let Ok(scorer) = ProbabilisticScorer::read(
                &mut cursor,
                (
                    params.clone(),
                    Arc::clone(&network_graph),
                    self.logger.clone(),
                ),
            ) {
                return scorer;
            }
        }
        ProbabilisticScorer::new(params, network_graph, self.logger.clone())
    }

    pub fn persist_scorer(
        &self,
        scorer: &ProbabilisticScorer<Arc<NetworkGraph>, Arc<FilesystemLogger>>,
    ) -> std::io::Result<()> {
        self.store.persist("scorer", scorer)
    }

    pub fn persist_graph(&self, graph: &NetworkGraph) -> std::io::Result<()> {
        self.store.persist("graph", graph)
    }

    /// Read `ChannelMonitor`s from disk.
    pub fn read_channelmonitors<Signer: Sign, K: Deref>(
        &self,
        keys_manager: K,
    ) -> Result<Vec<(BlockHash, ChannelMonitor<Signer>)>, std::io::Error>
    where
        K::Target: KeysInterface<Signer = Signer> + Sized,
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
                    if channel_monitor.get_funding_txo().0.txid != txid.unwrap()
                        || channel_monitor.get_funding_txo().0.index != index.unwrap()
                    {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "ChannelMonitor was stored in the wrong file",
                        ));
                    }
                    res.push((blockhash, channel_monitor));
                }
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to deserialize ChannelMonitor: {}", e),
                    ))
                }
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
