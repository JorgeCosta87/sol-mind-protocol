use litesvm::LiteSVM;
use sol_mind_protocol_client::types::{Fee, FeesStructure, Operation};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;

use crate::setup::AccountHelper;

use super::instructions::Instructions;
use super::test_data::*;

pub struct TestFixture {
    pub svm: LiteSVM,
    pub program_id_sol_mind: Pubkey,
    pub program_id_nft_operations: Pubkey,
    pub payer: Keypair,
    pub admin_1: Keypair,
    pub admin_2: Keypair,
    pub project_owner: Keypair,
    pub project_authority_1: Keypair,
    pub project_authority_2: Keypair,
    pub treasury: Pubkey,
    pub collection: Keypair,
}

impl TestFixture {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new().with_sysvars();
        let payer = Keypair::new();
        let admin_1 = Keypair::new();
        let admin_2 = Keypair::new();
        let project_owner = Keypair::new();
        let project_authority_1 = Keypair::new();
        let project_authority_2 = Keypair::new();
        let treasury: Pubkey = Keypair::new().pubkey();
        let collection = Keypair::new();

        let program_id_sol_mind = utils::deploy_program_from_keypair(
            &mut svm,
            SOL_MIND_PROTOCOL_KEYPAIR_PATH,
            SOL_MIND_PROTOCOL_SO_PATH,
        );
        let program_id_nft_operations = utils::deploy_program_from_keypair(
            &mut svm,
            NFT_OPERATIONS_KEYPAIR_PATH,
            NFT_OPERATIONS_SO_PATH,
        );

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund payer");

        svm.airdrop(&project_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund project owner");

        Self {
            svm,
            program_id_sol_mind,
            program_id_nft_operations,
            payer,
            admin_1,
            admin_2,
            project_owner,
            project_authority_1,
            project_authority_2,
            treasury,
            collection,
        }
    }

    pub fn with_metaplex_core_program(mut self) -> Self {
        let mpl_core_id =
            Pubkey::from_str(MPL_CORE_PROGRAM_ID).expect("Invalid MPL Core program ID");

        utils::deploy_program_from_id(&mut self.svm, mpl_core_id, MPL_CORE_PROGRAM_SO_PATH);

        self
    }

    pub fn with_initialize_protocol(mut self) -> Self {
        let fees = default_fees_structure();

        Instructions::initialize_protocol(
            &mut self.svm,
            vec![self.admin_1.pubkey(), self.admin_2.pubkey()],
            vec![self.admin_2.pubkey()],
            fees,
            self.payer.pubkey(),
            &[&self.payer.insecure_clone()],
        )
        .expect("Failed to initialize protocol");

        self
    }

    pub fn with_project_created(mut self, project_id: u64) -> Self {
        Instructions::create_project(
            &mut self.svm,
            project_id,
            DEFAULT_PROJECT_NAME.to_string(),
            DEFAULT_PROJECT_DESCRIPTION.to_string(),
            self.project_owner.pubkey(),
            vec![
                self.project_authority_1.pubkey(),
                self.project_authority_2.pubkey(),
            ],
            self.payer.pubkey(),
            &[
                &self.project_owner.insecure_clone(),
                &self.payer.insecure_clone(),
            ],
        )
        .expect("Failed to create project");

        self
    }

    pub fn with_initialize_project(mut self, project_id: u64) -> Self {
        Instructions::create_project(
            &mut self.svm,
            project_id,
            DEFAULT_PROJECT_NAME.to_string(),
            DEFAULT_PROJECT_DESCRIPTION.to_string(),
            self.project_owner.pubkey(),
            vec![
                self.project_authority_1.pubkey(),
                self.project_authority_2.pubkey(),
            ],
            self.payer.pubkey(),
            &[
                &self.project_owner.insecure_clone(),
                &self.payer.insecure_clone(),
            ],
        )
        .expect("Failed to initialize project");

        self
    }

    pub fn with_update_fees(mut self, fees: FeesStructure) -> Self {
        Instructions::update_fees(
            &mut self.svm,
            fees,
            self.admin_1.pubkey(),
            self.payer.pubkey(),
            &[&self.admin_1.insecure_clone()],
        )
        .expect("Failed to update fees");

        self
    }

    pub fn with_update_single_fee(mut self, operation: Operation, fee: Fee) -> Self {
        Instructions::update_single_fee(
            &mut self.svm,
            operation,
            fee,
            self.admin_1.pubkey(),
            self.payer.pubkey(),
            &[&self.admin_1.insecure_clone()],
        )
        .expect("Failed to update single fee");

        self
    }

    pub fn with_create_minter_config(
        mut self,
        project_id: u64,
        collection: Option<&Keypair>,
    ) -> Self {
        let mut signers = vec![
            self.payer.insecure_clone(),
            self.project_authority_1.insecure_clone(),
        ];

        if let Some(ref collection_keypair) = collection {
            signers.push(collection_keypair.insecure_clone());
        }

        let signing_keypairs: Vec<&Keypair> = signers.iter().collect();

        Instructions::create_minter_config(
            &mut self.svm,
            MINTER_NAME.to_string(),
            MINT_PRICE,
            MAX_SUPPLY,
            None,
            None,
            collection.as_ref().map(|_| COLLECTION_URI.to_string()),
            project_id,
            self.project_owner.pubkey(),
            self.payer.pubkey(),
            self.project_authority_1.pubkey(),
            collection.map(|k| k.pubkey()),
            &signing_keypairs,
        )
        .expect("Failed to create minter config");

        self
    }

    pub fn with_minted_asset(
        mut self,
        project_id: u64,
        asset_owner: &Keypair,
        mint: &Keypair,
        collection: Option<Pubkey>,
    ) -> Self {
        Instructions::mint_asset(
            &mut self.svm,
            MINTER_NAME,
            Some(ASSET_NAME.to_string()),
            Some(ASSET_URI.to_string()),
            None,
            project_id,
            self.project_owner.pubkey(),
            self.payer.pubkey(),
            asset_owner.pubkey(),
            mint.pubkey(),
            self.project_authority_1.pubkey(),
            collection,
            &[
                &self.payer.insecure_clone(),
                &asset_owner.insecure_clone(),
                &self.project_authority_1.insecure_clone(),
                &mint.insecure_clone(),
            ],
        )
        .expect("Failed to mint asset");

        self
    }

    pub fn with_create_trade_hub(mut self, project_id: u64) -> Self {
        Instructions::create_trade_hub(
            &mut self.svm,
            TRADE_HUB_NAME.to_string(),
            TRADE_HUB_FEE_BPS,
            project_id,
            self.project_owner.pubkey(),
            self.payer.pubkey(),
            self.project_authority_1.pubkey(),
            &[
                &self.payer.insecure_clone(),
                &self.project_authority_1.insecure_clone(),
            ],
        )
        .expect("Failed to create trade hub");

        self
    }

    pub fn with_list_asset(
        mut self,
        project_id: u64,
        mint: Pubkey,
        asset_owner: &Keypair,
        collection: Option<Pubkey>,
    ) -> Self {
        let project_config_pda =
            AccountHelper::find_project_pda(&self.project_owner.pubkey(), project_id).0;

        Instructions::list_asset(
            &mut self.svm,
            LISTING_PRICE,
            self.payer.pubkey(),
            &asset_owner.pubkey(),
            &mint,
            TRADE_HUB_NAME,
            &project_config_pda,
            collection,
            &[&self.payer.insecure_clone(), &asset_owner.insecure_clone()],
        )
        .expect("Failed to list asset");

        self
    }

    pub fn purchase_asset(
        mut self,
        project_id: u64,
        buyer: &Keypair,
        mint: Pubkey,
        asset_owner: &Keypair,
        collection: Option<Pubkey>,
    ) -> Self {
        let project_config_pda =
            AccountHelper::find_project_pda(&self.project_owner.pubkey(), project_id).0;

        Instructions::purchase_asset(
            &mut self.svm,
            buyer.pubkey(),
            &asset_owner.pubkey(),
            &mint,
            TRADE_HUB_NAME,
            &project_config_pda,
            collection,
            &[&buyer.insecure_clone()],
        )
        .expect("Failed to list asset");

        self
    }
}
