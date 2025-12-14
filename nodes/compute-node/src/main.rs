// Main entry point
mod config;
mod adapters;
mod services;

use anyhow::Result;
use config::Config;
use adapters::SolanaAdapter;
use services::NodeRegistryService;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    println!("Starting compute node...");
    println!("Node pubkey: {}", config.node_pubkey());

    let keypair = config.node_keypair.insecure_clone();
    let solana_adapter = SolanaAdapter::new(config.rpc_url.clone(), keypair).await;

    let node_registry_service = NodeRegistryService::new(solana_adapter);

    if let Some(signature) = node_registry_service.check_and_claim_pending_node(&config.node_pubkey()).await? {
        println!("✅ Claimed compute node: {}", signature);
    } else {
        println!("ℹ️  Compute node not pending or not found");
    }

    Ok(())
}
