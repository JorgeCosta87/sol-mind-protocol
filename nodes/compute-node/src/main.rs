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
    println!("RPC URL: {}", config.rpc_url);
    println!("RPC WebSocket URL: {}", config.rpc_websocket_url);

    let solana_adapter = SolanaAdapter::new(
        config.rpc_url.clone(), config.rpc_websocket_url.clone(),
         config.node_keypair.insecure_clone()
    ).await;

    startup_checks(&config, &solana_adapter).await?;

    let node_registry_service = NodeRegistryService::new(solana_adapter);
    node_registry_service.check_and_claim_pending_node(&config.node_pubkey()).await?;
    
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    Ok(())
}


async fn startup_checks(config: &Config, solana_adapter: &SolanaAdapter) -> Result<()> {
    let balance = solana_adapter.get_balance(&config.node_pubkey()).await?;
    println!("Balance: {}", balance);
    if balance < 1_000_000 {
        anyhow::bail!("Node requires at least 0.001 SOL deposited on node wallet: {}", config.node_pubkey());
    }

    Ok(())
}