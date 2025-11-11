use litesvm::LiteSVM;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::{Keypair, Signer, read_keypair_file}, transaction::Transaction
};
use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID;
use solana_pubkey::Pubkey;
use sol_mind_protocol_client::{
    generated::SOL_MIND_PROTOCOL_ID,
    generated::instructions::InitializeProjectBuilder,
};

mod constants;
use constants::*;

pub struct TestFixture {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub treasury: Pubkey,
    pub program_id: Pubkey,
}

impl TestFixture {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to fund payer");

        let program_keypair = read_keypair_file("target/deploy/sol_mind_protocol-keypair.json")
            .expect("Program keypair file not found");
        let program_id = program_keypair.pubkey();

        assert_eq!(program_id, SOL_MIND_PROTOCOL_ID, "Program ID mismatch!");

        let program_bytes = include_bytes!("../target/deploy/sol_mind_protocol.so");

        svm.add_program(program_id, program_bytes)
            .expect("Failed to deploy program");

        assert!(svm.get_account(&program_id).is_some(), "Program account not created");
        assert!(svm.get_account(&program_id).unwrap().executable, "Program not executable");

        let treasury = Keypair::new().pubkey();

        Self {
            svm,
            payer,
            treasury,
            program_id,
        }
    }

    pub fn find_project_pda(&self, owner: &Pubkey, project_id: u64) -> (Pubkey, u8) {
        Pubkey::try_find_program_address(
            &[
                b"project",
                owner.as_ref(),
                &project_id.to_le_bytes(),
            ],
            &self.program_id,
        ).unwrap()
    }
}

#[test]
fn test_initialize_project() {
    let mut fixture = TestFixture::new();

    let (project_config_pda, _) = fixture.find_project_pda(
        &fixture.payer.pubkey(), PROJECT_1_ID
    );

    let instruction = InitializeProjectBuilder::new()
        .owner(fixture.payer.pubkey())
        .project_config(project_config_pda)
        .system_program(SYSTEM_PROGRAM_ID)
        .project_id(PROJECT_1_ID)
        .name("Test Project".to_string())
        .description("Test Description".to_string())
        .treasury(fixture.treasury)
        .authorities(vec![fixture.payer.pubkey()])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&fixture.payer.pubkey()),
        &[&fixture.payer],
        fixture.svm.latest_blockhash(),
    );

    let result = fixture.svm.send_transaction(tx);
    
    match result {
        Ok(result) => {
            println!("Transaction logs: {:?}", result.logs);
        }
        Err(e) => {
            println!("Transaction error: {:?}", e);
            panic!("Transaction failed: {:?}", e);
        }
    }
}

