use anyhow::Result;
use bitcoin::util::bip32::ExtendedPubKey;
use bitcoin::Network;
use lightning_signer::bitcoin;
use vls_protocol_client::{DynSigner, SpendableKeysInterface};

pub mod util;
pub mod vls;

pub const SIGNER_NAMES: [&str; 2] = ["local", "grpc"];

/// Get the keys manager and the sweep address
pub async fn get_keys_manager(
    name: &str,
    network: Network,
    data_dir: String,
) -> Result<(
    Box<dyn SpendableKeysInterface<Signer = DynSigner>>,
    ExtendedPubKey,
)> {
    let (manager, xpub) = match name {
        "local" => vls::make_null_signer(network, data_dir).await,
        // "grpc" => {
        // 	vls::make_grpc_signer(
        // 		network,
        // 		data_dir,
        // 	)
        // 	.await
        // }
        _ => anyhow::bail!("not found"),
    };

    Ok((manager, xpub))
}
