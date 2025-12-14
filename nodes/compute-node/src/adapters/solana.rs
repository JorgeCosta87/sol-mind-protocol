use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use sol_mind_protocol_client::dac_manager::{
    accounts::{fetch_maybe_compute_node_info, ComputeNodeInfo},
    instructions::ClaimComputeNodeBuilder,
    programs::DAC_MANAGER_ID,
    types::ComputeNodeStatus,
};
use std::sync::Arc;

pub struct SolanaAdapter {
    client: Arc<RpcClient>,
    keypair: solana_sdk::signature::Keypair,
}

impl SolanaAdapter {
    pub async fn new(rpc_url: String, keypair: solana_sdk::signature::Keypair) -> Arc<Self> {
        Arc::new(Self {
            client: Arc::new(
                RpcClient::new_with_commitment(
                    rpc_url,
                    CommitmentConfig::confirmed(),
                ),
            ),
            keypair,
        })
    }

    pub fn derive_compute_node_info_pda(
        &self,
        node_pubkey: &Pubkey,
    ) -> Result<Pubkey> {
        let (address, _) = Pubkey::try_find_program_address(
            &[b"compute_node", &node_pubkey.as_ref()],
            &DAC_MANAGER_ID,
        )
        .ok_or_else(|| anyhow::anyhow!("Failed to derive compute node address"))?;
        Ok(address)
    }

    pub async fn get_compute_node_info(
        &self,
        node_pubkey: &Pubkey,
    ) -> Result<Option<ComputeNodeInfo>> {
        let pda = self.derive_compute_node_info_pda(&node_pubkey.clone())?;
        self.get_compute_node_info_by_pda(pda).await
    }

    pub async fn get_compute_node_info_by_pda(&self, pda: Pubkey) -> Result<Option<ComputeNodeInfo>> {
        let maybe_account =    fetch_maybe_compute_node_info(&self.client, &pda)?;

        match maybe_account {
            sol_mind_protocol_client::shared::MaybeAccount::Exists(account) => {
                Ok(Some(account.data))
            }
            _ => Ok(None),
        }
    }

    pub async fn claim_node(
        &self,
        compute_node_address: &Pubkey,
        compute_node_info_pda: Pubkey,
        node_info_cid: String,
        payer: &Pubkey,
    ) -> Result<String> {
        let instruction = ClaimComputeNodeBuilder::new()
            .payer(payer.clone())
            .compute_node(compute_node_address.clone())
            .compute_node_info(compute_node_info_pda)
            .node_info_cid(node_info_cid)
            .instruction();

        let client = Arc::clone(&self.client);
        let keypair = self.keypair.insecure_clone();
        let signature = tokio::task::spawn_blocking(move || {
            let latest_blockhash = client.get_latest_blockhash()?;

            let mut transaction = Transaction::new_signed_with_payer(&[instruction], None, &[&keypair], latest_blockhash);
            transaction.sign(&[&keypair], latest_blockhash);

            client.send_and_confirm_transaction(&transaction)?;

            Ok::<_, anyhow::Error>(transaction.signatures[0].to_string())
        })
        .await??;

        Ok(signature)
    }
}


pub trait ComputeNodeInfoExt {
    fn is_pending(&self) -> bool;
    fn can_be_claimed(&self) -> bool;
}

impl ComputeNodeInfoExt for ComputeNodeInfo {
    fn is_pending(&self) -> bool {
        self.status == ComputeNodeStatus::Pending
    }

    fn can_be_claimed(&self) -> bool {
        self.is_pending()
    }
}
