use litesvm::LiteSVM;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;

use crate::setup::accounts::AccountHelper;
use crate::setup::constants::*;
use crate::setup::instructions::Instructions;

pub struct TestFixture {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub owner: Keypair,
    pub payer: Keypair,
    pub project_authority_1: Keypair,
    pub project_authority_2: Keypair,
    pub treasury: Pubkey,
}

impl TestFixture {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        let owner = Keypair::new();
        let payer = Keypair::new();
        let project_authority_1 = Keypair::new();
        let project_authority_2 = Keypair::new();
        let treasury: Pubkey = Keypair::new().pubkey();

        let program_id =
            utils::deploy_program_from_keypair(&mut svm, PROGRAM_KEYPAIR_PATH, PROGRAM_SO_PATH);

        svm.airdrop(&owner.pubkey(), 1 * LAMPORTS_PER_SOL)
            .expect("Failed to fund owner");
        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund payer");

        Self {
            svm,
            program_id,
            owner,
            payer,
            project_authority_1,
            project_authority_2,
            treasury,
        }
    }

    pub fn instructions(&self) -> Instructions {
        Instructions::new(self.program_id, self.owner.pubkey())
    }

    pub fn account_helper<'a>(&'a self) -> AccountHelper<'a> {
        AccountHelper::new(&self.svm, self.program_id)
    }

    pub fn with_metaplex_core_program(mut self) -> Self {
        let mpl_core_id =
            Pubkey::from_str(MPL_CORE_PROGRAM_ID).expect("Invalid MPL Core program ID");

        utils::deploy_program_from_id(&mut self.svm, mpl_core_id, MPL_CORE_PROGRAM_SO_PATH);

        self
    }

    pub fn with_initialize_project(mut self, project_id: u64) -> Self {
        let instructions = Instructions::new(self.program_id, self.owner.pubkey());

        instructions
            .initialize_project(
                &mut self.svm,
                project_id,
                DEFAULT_PROJECT_NAME.to_string(),
                DEFAULT_PROJECT_DESCRIPTION.to_string(),
                self.owner.pubkey(),
                self.treasury,
                vec![
                    self.project_authority_1.pubkey(),
                    self.project_authority_2.pubkey(),
                ],
                self.owner.pubkey(),
                &[&self.owner.insecure_clone()],
            )
            .expect("Failed to initialize project");

        self
    }

    pub fn with_minter_config(mut self, project_id: u64, collection: Option<&Keypair>) -> Self {
        if collection.is_some() {
            self = self.with_metaplex_core_program();
        }

        let instructions = Instructions::new(self.program_id, self.owner.pubkey());

        let mut signers = vec![
            self.payer.insecure_clone(),
            self.project_authority_1.insecure_clone(),
        ];

        if let Some(ref collection_keypair) = collection {
            signers.push(collection_keypair.insecure_clone());
        }

        let signing_keypairs: Vec<&Keypair> = signers.iter().collect();

        instructions
            .create_minter_config(
                &mut self.svm,
                MINTER_NAME.to_string(),
                MINT_PRICE,
                MAX_SUPPLY,
                None,
                None,
                collection.as_ref().map(|_| COLLECTION_URI.to_string()),
                project_id,
                self.payer.pubkey(),
                self.project_authority_1.pubkey(),
                collection.map(|k| k.pubkey()),
                &signing_keypairs,
            )
            .expect("Failed to create minter config");

        self
    }
}
