mod setup;

use sol_mind_protocol_client::types::{Fee, FeeType, FeesStructure, Operation};
use solana_sdk::signature::Keypair;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, signature::Signer};

use crate::setup::test_data::*;
use setup::{AccountHelper, Instructions, TestFixture};

#[test]
fn test_initialize_protocol() {
    let mut fixture = TestFixture::new();

    let admins = vec![fixture.admin_1.pubkey()];
    let whitelist_transfer_addrs = vec![fixture.admin_1.pubkey()];
    let fees = default_fees_structure();

    let result = Instructions::initialize_protocol(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        admins.clone(),
        whitelist_transfer_addrs.clone(),
        fees.clone(),
        fixture.payer.pubkey(),
        &[&fixture.payer.insecure_clone()],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let protocol_config =
                AccountHelper::get_protocol_config(&fixture.svm, &fixture.program_id_sol_mind);

            assert_eq!(protocol_config.admins, admins);
            assert_eq!(
                protocol_config.whitelist_transfer_addrs,
                whitelist_transfer_addrs
            );
            assert_eq!(
                protocol_config.fees.create_project.amount,
                fees.create_project.amount
            );
            assert_eq!(
                protocol_config.fees.create_project.fee_type,
                fees.create_project.fee_type
            );
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_create_project() {
    let mut fixture = TestFixture::new().with_initialize_protocol();

    let (protocol_config_pda, _) =
        AccountHelper::find_protocol_config_pda(&fixture.program_id_sol_mind);
    let protocol_initial_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

    let project_name = "Test project".to_string();
    let project_description = "Project description".to_string();
    let authorities = vec![
        fixture.project_authority_1.pubkey(),
        fixture.project_authority_2.pubkey(),
    ];

    let result = Instructions::create_project(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        PROJECT_1_ID,
        project_name.clone(),
        project_description.clone(),
        fixture.project_owner.pubkey(),
        authorities.clone(),
        fixture.payer.pubkey(),
        &[
            &fixture.project_owner.insecure_clone(),
            &fixture.payer.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let project_config = AccountHelper::get_project_config(
                &fixture.svm,
                &fixture.program_id_sol_mind,
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let protocol_config =
                AccountHelper::get_protocol_config(&fixture.svm, &fixture.program_id_sol_mind);

            let (project_config_pda, _) = AccountHelper::find_project_pda(
                &fixture.program_id_sol_mind,
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let (_, treasury_bump) =
                AccountHelper::find_treasury_pda(&fixture.program_id_sol_mind, &project_config_pda);

            let protocol_final_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

            assert_eq!(project_config.owner, fixture.project_owner.pubkey());
            assert_eq!(project_config.name, project_name);
            assert_eq!(project_config.description, project_description);
            assert_eq!(project_config.treasury_bump, treasury_bump);

            assert_eq!(
                protocol_final_balance,
                protocol_initial_balance + protocol_config.fees.create_project.amount
            )
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_update_fees() {
    let mut fixture = TestFixture::new().with_initialize_protocol();

    let new_fees = FeesStructure {
        create_project: Fee {
            amount: 2_000_000,
            fee_type: FeeType::Fixed,
        },
        create_minter_config: Fee {
            amount: 1_000_000,
            fee_type: FeeType::Fixed,
        },
        mint_asset: Fee {
            amount: 100_000,
            fee_type: FeeType::Fixed,
        },
        generic_operation: Fee {
            amount: 200_000,
            fee_type: FeeType::Fixed,
        },
    };

    let result = Instructions::update_fees(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        new_fees.clone(),
        fixture.admin_1.pubkey(),
        fixture.payer.pubkey(),
        &[
            &fixture.admin_1.insecure_clone(),
            &fixture.payer.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let protocol_config =
                AccountHelper::get_protocol_config(&fixture.svm, &fixture.program_id_sol_mind);

            assert_eq!(
                protocol_config.fees.create_project.amount,
                new_fees.create_project.amount
            );
            assert_eq!(
                protocol_config.fees.create_minter_config.amount,
                new_fees.create_minter_config.amount
            );
            assert_eq!(
                protocol_config.fees.mint_asset.amount,
                new_fees.mint_asset.amount
            );
            assert_eq!(
                protocol_config.fees.generic_operation.amount,
                new_fees.generic_operation.amount
            );
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_update_single_fee() {
    let mut fixture = TestFixture::new().with_initialize_protocol();

    let new_fee = Fee {
        amount: 3_000_000,
        fee_type: FeeType::Fixed,
    };

    let result = Instructions::update_single_fee(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        Operation::CreateProject,
        new_fee.clone(),
        fixture.admin_1.pubkey(),
        fixture.payer.pubkey(),
        &[
            &fixture.admin_1.insecure_clone(),
            &fixture.payer.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let protocol_config =
                AccountHelper::get_protocol_config(&fixture.svm, &fixture.program_id_sol_mind);

            assert_eq!(protocol_config.fees.create_project.amount, new_fee.amount);
            assert_eq!(
                protocol_config.fees.create_project.fee_type,
                new_fee.fee_type
            );
            assert_eq!(
                protocol_config.fees.mint_asset.amount,
                FEE_MINT_ASSET_AMOUNT
            );
            assert_eq!(
                protocol_config.fees.create_minter_config.amount,
                FEE_CREATE_MINTER_CONFIG_AMOUNT
            );
            assert_eq!(
                protocol_config.fees.generic_operation.amount,
                FEE_GENERIC_OPERATION_AMOUNT
            );
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_protocol_fees_transfer() {
    let mut fixture = TestFixture::new().with_initialize_protocol();

    let (protocol_config_pda, _) =
        AccountHelper::find_protocol_config_pda(&fixture.program_id_sol_mind);
    fixture
        .svm
        .airdrop(&protocol_config_pda, 5 * LAMPORTS_PER_SOL)
        .expect("Failed to fund protocol config");

    let initial_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

    let transfer_amount = 1 * LAMPORTS_PER_SOL;
    let destination = fixture.admin_2.pubkey();

    let result = Instructions::protocol_fees_transfer(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        transfer_amount,
        fixture.admin_1.pubkey(),
        destination,
        fixture.payer.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.admin_1.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let final_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);
            let destination_final_balance = utils::get_lamports(&fixture.svm, &destination);

            assert_eq!(final_balance, initial_balance - transfer_amount);
            assert_eq!(destination_final_balance, transfer_amount);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_protocol_fees_transfer_non_admin() {
    let mut fixture = TestFixture::new().with_initialize_protocol();

    let (protocol_config_pda, _) =
        AccountHelper::find_protocol_config_pda(&fixture.program_id_sol_mind);
    fixture
        .svm
        .airdrop(&protocol_config_pda, 5 * LAMPORTS_PER_SOL)
        .expect("Failed to fund protocol config");

    let initial_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

    let transfer_amount = 1 * LAMPORTS_PER_SOL;
    let destination = fixture.admin_2.pubkey();
    let non_admin = Keypair::new();

    let result = Instructions::protocol_fees_transfer(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        transfer_amount,
        non_admin.pubkey(),
        destination,
        fixture.payer.pubkey(),
        &[&fixture.payer.insecure_clone(), &non_admin.insecure_clone()],
    );

    match result {
        Ok(_) => {
            panic!("Transaction should have failed, non-admin cannot transfer fees");
        }
        Err(e) => {
            let error_string = format!("{:?}", e);
            assert!(
                error_string.contains("Unauthorized"),
                "Expected Unauthorized error, got: {:?}",
                e
            );

            let final_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);
            assert_eq!(
                final_balance, initial_balance,
                "Balance should not change on failed transfer"
            );
        }
    }
}

#[test]
fn test_protocol_fees_transfer_to_non_whitelisted_address() {
    let mut fixture = TestFixture::new().with_initialize_protocol();

    let (protocol_config_pda, _) =
        AccountHelper::find_protocol_config_pda(&fixture.program_id_sol_mind);
    fixture
        .svm
        .airdrop(&protocol_config_pda, 5 * LAMPORTS_PER_SOL)
        .expect("Failed to fund protocol config");

    let initial_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

    let transfer_amount = 1 * LAMPORTS_PER_SOL;
    let destination = Keypair::new().pubkey();

    let result = Instructions::protocol_fees_transfer(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        transfer_amount,
        fixture.admin_1.pubkey(),
        destination,
        fixture.payer.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.admin_1.insecure_clone(),
        ],
    );

    match result {
        Ok(_) => {
            panic!("Transaction should have failed, destination is not whitelisted");
        }
        Err(e) => {
            let error_string = format!("{:?}", e);
            assert!(
                error_string.contains("AddressNotWhiteListed"),
                "Expected AddressNotWhiteListed error, got: {:?}",
                e
            );

            let final_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);
            assert_eq!(
                final_balance, initial_balance,
                "Balance should not change on failed transfer"
            );
        }
    }
}

#[test]
fn test_project_fees_transfer() {
    let mut fixture = TestFixture::new()
        .with_initialize_protocol()
        .with_project_created(PROJECT_1_ID);

    let (project_config_pda, _) = AccountHelper::find_project_pda(
        &fixture.program_id_sol_mind,
        &fixture.project_owner.pubkey(),
        PROJECT_1_ID,
    );

    let (treasury_pda, _) =
        AccountHelper::find_treasury_pda(&fixture.program_id_sol_mind, &project_config_pda);

    fixture
        .svm
        .airdrop(&treasury_pda, 3 * LAMPORTS_PER_SOL)
        .expect("Failed to fund treasury");

    let initial_balance = utils::get_lamports(&fixture.svm, &treasury_pda);

    let transfer_amount = 1 * LAMPORTS_PER_SOL;
    let destination = fixture.project_authority_1.pubkey();

    let result = Instructions::transfer_project_fees(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        transfer_amount,
        fixture.project_owner.pubkey(),
        destination,
        PROJECT_1_ID,
        fixture.payer.pubkey(),
        &[
            &fixture.project_owner.insecure_clone(),
            &fixture.payer.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let final_balance = utils::get_lamports(&fixture.svm, &treasury_pda);
            let destination_final_balance = utils::get_lamports(&fixture.svm, &destination);

            assert_eq!(final_balance, initial_balance - transfer_amount);
            assert_eq!(destination_final_balance, transfer_amount);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_project_fees_transfer_by_non_owner() {
    let mut fixture = TestFixture::new()
        .with_initialize_protocol()
        .with_project_created(PROJECT_1_ID);

    let (project_config_pda, _) = AccountHelper::find_project_pda(
        &fixture.program_id_sol_mind,
        &fixture.project_owner.pubkey(),
        PROJECT_1_ID,
    );

    let (treasury_pda, _) =
        AccountHelper::find_treasury_pda(&fixture.program_id_sol_mind, &project_config_pda);

    fixture
        .svm
        .airdrop(&treasury_pda, 3 * LAMPORTS_PER_SOL)
        .expect("Failed to fund treasury");

    let initial_balance = utils::get_lamports(&fixture.svm, &treasury_pda);

    let transfer_amount = 1 * LAMPORTS_PER_SOL;
    let destination = fixture.project_authority_1.pubkey();
    let non_owner = Keypair::new();

    let result = Instructions::transfer_project_fees(
        &mut fixture.svm,
        &fixture.program_id_sol_mind,
        transfer_amount,
        non_owner.pubkey(),
        destination,
        PROJECT_1_ID,
        fixture.payer.pubkey(),
        &[&non_owner.insecure_clone(), &fixture.payer.insecure_clone()],
    );

    match result {
        Ok(_) => {
            panic!("Transaction should have failed, non-owner cannot transfer fees");
        }
        Err(e) => {
            let error_string = format!("{:?}", e);
            assert!(
                error_string.contains("AccountNotInitialized."),
                "Expected AccountNotInitialized, since the PDA seed has the owner, got: {:?}",
                e
            );

            let final_balance = utils::get_lamports(&fixture.svm, &treasury_pda);
            assert_eq!(
                final_balance, initial_balance,
                "Balance should not change on failed transfer"
            );
        }
    }
}
