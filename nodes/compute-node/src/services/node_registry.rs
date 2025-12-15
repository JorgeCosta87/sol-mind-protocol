use anyhow::Result;
use tokio::sync::mpsc;
use crate::adapters::{SolanaAdapter, ComputeNodeInfoExt};
use solana_sdk::pubkey::Pubkey;
use sol_mind_protocol_client::dac_manager::accounts::ComputeNodeInfo;
use std::sync::Arc;

pub struct NodeRegistryService {
    adapter: Arc<SolanaAdapter>,
}

impl NodeRegistryService {
    pub fn new(adapter: Arc<SolanaAdapter>) -> Self {
        Self { adapter }
    }

    pub async fn check_and_claim_pending_node(
        &self,
        node_pubkey: &Pubkey,
    ) -> Result<()> {
        match self.adapter.get_compute_node_info_account(node_pubkey).await? {
            Some(compute_node) => {
                self.claim_node(node_pubkey, &compute_node).await?;
                return Ok(());
            }
            None => {
                println!("Compute node not registered, watching for register event...");
                self.start_watching_compute_node_info_accounts(node_pubkey);
                return Ok(());
            }
        }
    }

    async fn claim_node(
        &self,
        node_pubkey: &Pubkey,
        compute_node_info: &ComputeNodeInfo,
    ) -> Result<Option<String>> {
        if !compute_node_info.can_be_claimed() {
            println!("Compute node claimed, current status: {:?}, owner: {:?},", compute_node_info.status, compute_node_info.owner);
            return Ok(None);
        }
        let node_info_cid = "teste".to_string();
        let signature = self
            .adapter
            .claim_node(
                node_pubkey,
                node_info_cid,
            )
            .await?;

        println!("Claimed compute node: {:?}", signature);
        println!("Compute node info: {:#?}", compute_node_info);
        Ok(Some(signature))
    }

    pub fn start_watching_compute_node_info_accounts(
        &self, node_pubkey: &Pubkey
    ) -> tokio::task::JoinHandle<Result<()>> {
        let adapter = Arc::clone(&self.adapter);
        let node_pubkey = *node_pubkey;

        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel::<ComputeNodeInfo>(16);
            
            let watch_handle = adapter.watch_compute_node_info_accounts(&node_pubkey, tx);

            println!("Watching compute node info accounts...");
            loop {
                tokio::select! {
                    result = rx.recv() => {
                        match result {
                            Some(compute_node_info) => {
                                println!("Compute node info: {:?}", compute_node_info);

                                if compute_node_info.can_be_claimed() {
                                    let signature = adapter
                                        .claim_node(
                                            &node_pubkey,
                                            "teste".to_string(),
                                        )
                                        .await;

                                    match signature {
                                        Ok(sig) => {
                                            println!("Claimed compute node: {:?}", sig);
                                            watch_handle.abort();
                                            break;
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to claim node: {}", e);
                                        }
                                    }
                                }
                            }
                            None => {
                                println!("Watch channel closed");
                                break;
                            }
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("Closing watch compute node info accounts...");
                        watch_handle.abort();
                        break;
                    }
                }
            }

            Ok(())
        })
    }
}
