use litesvm::{
    types::{TransactionMetadata, TransactionResult},
    LiteSVM,
};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};

pub fn deploy_program_from_keypair(svm: &mut LiteSVM, keypair_path: &str, so_path: &str) -> Pubkey {
    let program_keypair = read_keypair_file(keypair_path).expect("Failed to read keypair file");
    let program_id = program_keypair.pubkey();

    deploy_program_internal(svm, program_id, so_path)
}

pub fn deploy_program_from_id(svm: &mut LiteSVM, program_id: Pubkey, so_path: &str) -> Pubkey {
    deploy_program_internal(svm, program_id, so_path)
}

fn deploy_program_internal(svm: &mut LiteSVM, program_id: Pubkey, so_path: &str) -> Pubkey {
    svm.add_program_from_file(program_id, so_path)
        .expect("Failed to deploy program from file");

    assert!(
        svm.get_account(&program_id).is_some(),
        "Program account not created"
    );
    assert!(
        svm.get_account(&program_id).unwrap().executable,
        "Program not executable"
    );

    program_id
}

pub fn print_transaction_logs(result: &TransactionMetadata) {
    println!("\nTransaction logs:");
    for log in &result.logs {
        println!("  {}", log);
    }
}

pub fn send_transaction(
    svm: &mut LiteSVM,
    instructions: &[Instruction],
    payer: &Pubkey,
    signing_keypairs: &[&Keypair],
) -> TransactionResult {
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(payer),
        signing_keypairs,
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx)
}
