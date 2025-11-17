use litesvm::LiteSVM;
use sol_mind_protocol_client::generated::types::{Fee, FeesStructure, Operation};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;

use crate::setup::instructions::Instructions;
use crate::setup::test_data::*;

pub struct TestFixture {
    pub svm: LiteSVM,
    pub program_id: Pubkey,
    pub payer: Keypair,
    pub admin_1: Keypair,
    pub admin_2: Keypair,
    pub project_owner: Keypair,
    pub project_authority_1: Keypair,
    pub project_authority_2: Keypair,
}

impl TestFixture {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();
        let admin_1 = Keypair::new();
        let admin_2 = Keypair::new();
        let project_owner = Keypair::new();
        let project_authority_1 = Keypair::new();
        let project_authority_2 = Keypair::new();

        let program_id =
            utils::deploy_program_from_keypair(&mut svm, PROGRAM_KEYPAIR_PATH, PROGRAM_SO_PATH);

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund payer");

        svm.airdrop(&project_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund project owner");

        Self {
            svm,
            payer,
            program_id,
            admin_1,
            admin_2,
            project_owner,
            project_authority_1,
            project_authority_2,
        }
    }

    pub fn with_metaplex_core_program(mut self) -> Self {
        let mpl_core_id =
            Pubkey::from_str(MPL_CORE_PROGRAM_ID).expect("Invalid MPL Core program ID");

        utils::deploy_program_from_id(&mut self.svm, mpl_core_id, MPL_CORE_PROGRAM_SO_PATH);

        self
    }

    pub fn with_initialize_protocol(mut self) -> Self {
        let fees = crate::setup::test_data::default_fees_structure();

        Instructions::initialize_protocol(
            &mut self.svm,
            &self.program_id,
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
            &self.program_id,
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

    pub fn with_update_fees(mut self, fees: FeesStructure) -> Self {
        Instructions::update_fees(
            &mut self.svm,
            &self.program_id,
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
            &self.program_id,
            operation,
            fee,
            self.admin_1.pubkey(),
            self.payer.pubkey(),
            &[&self.admin_1.insecure_clone()],
        )
        .expect("Failed to update single fee");

        self
    }
}
