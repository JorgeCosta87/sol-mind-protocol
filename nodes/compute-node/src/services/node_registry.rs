use anyhow::Result;
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
    ) -> Result<Option<String>> {
        let compute_node_info_pda = self.adapter.derive_compute_node_info_pda(node_pubkey)?;
        let compute_node = self
            .adapter
            .get_compute_node_info(node_pubkey)
            .await?;

        let Some(compute_node) = compute_node else {
            return Ok(None); 
        };

        Ok(self.claim_node(node_pubkey, &compute_node, &compute_node_info_pda).await?)
    }

   async fn claim_node(
    &self,
    node_pubkey: &Pubkey,
    compute_node_info: &ComputeNodeInfo,
    compute_node_info_pda: &Pubkey,
   ) -> Result<Option<String>> {
        if !compute_node_info.can_be_claimed() {
            return Ok(None);
        }

        let node_info_cid = "teste".to_string();

        let signature = self
            .adapter
            .claim_node(
                node_pubkey,
                compute_node_info_pda.clone(),
                node_info_cid,
                node_pubkey,
            )
            .await?;
        
        Ok(Some(signature))
    }
}
