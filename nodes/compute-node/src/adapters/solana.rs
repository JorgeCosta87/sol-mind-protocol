use anyhow::Result;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction
};
use sol_mind_protocol_client::dac_manager::{
    accounts::{fetch_maybe_compute_node_info, ComputeNodeInfo, COMPUTE_NODE_INFO_DISCRIMINATOR},
    instructions::ClaimComputeNodeBuilder,
    programs::DAC_MANAGER_ID,
    types::ComputeNodeStatus,
};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio_stream::StreamExt;
use base64::Engine;

pub struct SolanaAdapter {
    client: Arc<RpcClient>,
    ws_client: Arc<PubsubClient>,
    keypair: Keypair,
}


impl SolanaAdapter {
    pub async fn new(rpc_url: String, rpc_websocket_url: String, keypair: Keypair) -> Arc<Self> {
        Arc::new(Self {
            client: Arc::new(
                RpcClient::new_with_commitment(
                    rpc_url,
                    CommitmentConfig::confirmed(),
                ),
            ),
            ws_client: Arc::new(match PubsubClient::new(&rpc_websocket_url).await {
                Ok(client) => client,
                Err(e) => {
                    panic!("Failed to create WebSocket client: {}", e);
                }
            }),
            keypair,
        })
    }

    pub fn payer_pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let client = Arc::clone(&self.client);
        let pubkey = *pubkey;
        
        tokio::task::spawn_blocking(move || {
            client.get_balance(&pubkey).map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))
        })
        .await?
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

    pub async fn get_compute_node_info_account(
        &self,
        node_pubkey: &Pubkey,
    ) -> Result<Option<ComputeNodeInfo>> {
        let pda = self.derive_compute_node_info_pda(&node_pubkey.clone())?;
        self.get_compute_node_info_by_addres(pda).await
    }

    pub async fn get_compute_node_info_by_addres(&self, address: Pubkey) -> Result<Option<ComputeNodeInfo>> {
        let maybe_account =    fetch_maybe_compute_node_info(&self.client, &address)?;

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
        node_info_cid: String,
    ) -> Result<String> {
        let compute_node_info_pda = self.derive_compute_node_info_pda(compute_node_address)?;

        let instruction = ClaimComputeNodeBuilder::new()
            .payer(self.payer_pubkey())
            .compute_node(compute_node_address.clone())
            .compute_node_info(compute_node_info_pda)
            .node_info_cid(node_info_cid)
            .instruction();

        let signature = self.send_and_confirm_transaction(
            &[instruction],
            self.payer_pubkey(),
            &[&self.keypair],
        ).await?;

        Ok(signature)
    }

    pub async fn send_and_confirm_transaction(
        &self,
        instructions: &[Instruction],
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> Result<String> {
        let client = Arc::clone(&self.client);
        let instructions = instructions.to_vec();
        let signing_keypairs: Vec<Keypair> = signing_keypairs.iter().map(|k| k.insecure_clone()).collect();

        tokio::task::spawn_blocking(move || { // Probably codama generate async clients!
            let latest_blockhash = client.get_latest_blockhash()?;

            let signing_keypairs_refs: Vec<&Keypair> = signing_keypairs.iter().collect();
            let transaction = Transaction::new_signed_with_payer(
                &instructions,
                Some(&payer),
                &signing_keypairs_refs,
                latest_blockhash,
            );

            client.send_and_confirm_transaction(&transaction)?;
            Ok(transaction.signatures[0].to_string())
        })
        .await?
    }

    pub fn watch_compute_node_info_accounts(
        &self,
        node_pubkey: &Pubkey,
        tx: mpsc::Sender<ComputeNodeInfo>,
    ) -> tokio::task::JoinHandle<()> {
        let ws_client = Arc::clone(&self.ws_client);

        let filters = vec![
            RpcFilterType::Memcmp(Memcmp::new(
                0,
                MemcmpEncodedBytes::Bytes(COMPUTE_NODE_INFO_DISCRIMINATOR.to_vec()),
            )),
            RpcFilterType::Memcmp(Memcmp::new(
                8 + 32,
                MemcmpEncodedBytes::Bytes(node_pubkey.as_ref().to_vec()),
            )),
        ];

        let config = RpcProgramAccountsConfig {
            filters: Some(filters),
            account_config: RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::confirmed()),
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                min_context_slot: None,
            },
            with_context: None,
            sort_results: None,
        };

        tokio::spawn(async move {
            let (mut stream, _) = match ws_client
                .program_subscribe(&DAC_MANAGER_ID, Some(config))
                .await
            {
                Ok(sub) => sub,
                Err(e) => {
                    panic!("Failed to subscribe to compute node info accounts: {}", e);
                }
            };

            while let Some(account) = stream.next().await {
                if let solana_account_decoder::UiAccountData::Binary(data_str, _) =
                    &account.value.account.data
                {
                    if let Ok(decoded_bytes) =
                        base64::engine::general_purpose::STANDARD.decode(data_str)
                    {
                        if let Ok(compute_node) = ComputeNodeInfo::from_bytes(&decoded_bytes) {
                            if tx.send(compute_node).await.is_err() {
                                println!("Receiver dropped, stopping watch");
                            }
                        }
                    }
                }
            }
        })
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
