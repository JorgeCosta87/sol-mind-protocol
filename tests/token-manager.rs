mod setup;

use anchor_lang::AnchorSerialize;
use mpl_core::types::{Creator, Plugin, PluginAuthority, PluginAuthorityPair, Royalties};
use solana_program::pubkey::Pubkey as ProgramPubkey;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};

use setup::test_data::*;
use setup::{AccountHelper, Instructions, TestFixture};
use utils::MplUtils;

#[test]
fn test_create_minter_config_without_collection() {
    let mut fixture = TestFixture::new()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID);

    let (protocol_config_pda, _) =
        AccountHelper::find_protocol_config_pda(&fixture.program_id_sol_mind);
    let protocol_initial_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

    let result = Instructions::create_minter_config(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        None,
        None,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        fixture.project_authority_1.pubkey(),
        None,
        &[
            &fixture.payer.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let protocol_config =
                AccountHelper::get_protocol_config(&fixture.svm, &fixture.program_id_sol_mind);
            let (project_config_pda, _) = AccountHelper::find_project_pda(
                &fixture.program_id_sol_mind,
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let minter_config = AccountHelper::get_minter_config(
                &fixture.svm,
                &fixture.program_id_token_manager,
                &project_config_pda,
                &MINTER_NAME,
            );
            let protocol_final_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

            assert_eq!(minter_config.name, MINTER_NAME);
            assert_eq!(minter_config.mint_price, MINT_PRICE);
            assert_eq!(minter_config.max_supply, MAX_SUPPLY);
            assert_eq!(minter_config.mints_counter, 0);
            assert_eq!(minter_config.collection, None);
            assert_eq!(minter_config.assets_config, None);
            assert_eq!(
                protocol_final_balance,
                protocol_initial_balance + protocol_config.fees.create_minter_config.amount
            );
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_create_minter_config_with_unauthorized_authority() {
    let mut fixture = TestFixture::new()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID);

    let unauthorized_authority = Keypair::new();

    fixture
        .svm
        .airdrop(&unauthorized_authority.pubkey(), 1 * LAMPORTS_PER_SOL)
        .expect("Failed to fund unauthorized authority");

    let result = Instructions::create_minter_config(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        None,
        None,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        unauthorized_authority.pubkey(),
        None,
        &[
            &fixture.payer.insecure_clone(),
            &unauthorized_authority.insecure_clone(),
        ],
    );

    assert!(
        result.is_err(),
        "Transaction should have failed with unauthorized authority, but it succeeded"
    );

    if let Err(e) = result {
        println!("\nExpected transaction failure unauthorized authority:");
        println!("Error: {:?}", e);

        let error_string = format!("{:?}", e);
        assert!(
            error_string.contains("Unauthorized"),
            "Error should indicate unauthorized access, got: {:?}",
            e
        );
    }
}

#[test]
fn test_create_minter_config_with_collection() {
    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID);

    let collection = Keypair::new();

    let result = Instructions::create_minter_config(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        None,
        Some(COLLECTION_URI.to_string()),
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        fixture.project_authority_1.pubkey(),
        Some(collection.pubkey()),
        &[
            &fixture.payer.insecure_clone(),
            &collection.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let (project_config_pda, _) = AccountHelper::find_project_pda(
                &fixture.program_id_sol_mind,
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let minter_config = AccountHelper::get_minter_config(
                &fixture.svm,
                &fixture.program_id_token_manager,
                &project_config_pda,
                &MINTER_NAME,
            );

            assert_eq!(minter_config.name, MINTER_NAME);
            assert_eq!(minter_config.mint_price, MINT_PRICE);
            assert_eq!(minter_config.max_supply, MAX_SUPPLY);
            assert_eq!(minter_config.mints_counter, 0);
            assert_eq!(minter_config.collection.unwrap(), collection.pubkey());
            assert_eq!(minter_config.assets_config, None);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_create_minter_config_with_collection_with_plugins() {
    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID);

    let collection = Keypair::new();

    let royalties: Royalties = Royalties {
        basis_points: 100,
        creators: vec![Creator {
            address: ProgramPubkey::from(fixture.treasury.to_bytes()),
            percentage: 100,
        }],
        rule_set: mpl_core::types::RuleSet::None,
    };

    let plugin_authority = PluginAuthorityPair {
        plugin: Plugin::Royalties(royalties),
        authority: Some(PluginAuthority::None),
    };

    let plugins: Option<Vec<Vec<u8>>> = Some(
        vec![plugin_authority]
            .iter()
            .map(|pair| {
                let mut bytes = Vec::new();
                pair.serialize(&mut bytes)
                    .expect("Failed to serialize plugin");
                bytes
            })
            .collect(),
    );

    let result = Instructions::create_minter_config(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        plugins,
        Some(COLLECTION_URI.to_string()),
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        fixture.project_authority_1.pubkey(),
        Some(collection.pubkey()),
        &[
            &fixture.payer.insecure_clone(),
            &collection.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let _project_config = AccountHelper::get_project_config(
                &fixture.svm,
                &fixture.program_id_sol_mind,
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let (project_config_pda, _) = AccountHelper::find_project_pda(
                &fixture.program_id_sol_mind,
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let minter_config = AccountHelper::get_minter_config(
                &fixture.svm,
                &fixture.program_id_token_manager,
                &project_config_pda,
                &MINTER_NAME,
            );
            let asset = MplUtils::get_collection(&fixture.svm, &collection.pubkey());

            println!("plugins: {:?}", asset.plugin_list);

            assert_eq!(minter_config.name, MINTER_NAME);
            assert_eq!(minter_config.mint_price, MINT_PRICE);
            assert_eq!(minter_config.max_supply, MAX_SUPPLY);
            assert_eq!(minter_config.mints_counter, 0);
            assert_eq!(minter_config.collection.unwrap(), collection.pubkey());
            assert_eq!(minter_config.assets_config, None);

            assert_ne!(asset.plugin_list.royalties, None);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_mint_asset_without_assets_config_and_collection() {
    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_minter_config(PROJECT_1_ID, None);

    let (protocol_config_pda, _) =
        AccountHelper::find_protocol_config_pda(&fixture.program_id_sol_mind);
    let protocol_initial_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let result = Instructions::mint_asset(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME,
        Some(ASSET_NAME.to_string()),
        Some(ASSET_URI.to_string()),
        None,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        asset_owner.pubkey(),
        mint.pubkey(),
        fixture.project_authority_1.pubkey(),
        None,
        &[
            &fixture.payer.insecure_clone(),
            &asset_owner.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
            &mint.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let protocol_config =
                AccountHelper::get_protocol_config(&fixture.svm, &fixture.program_id_sol_mind);
            let (project_config_pda, _) = AccountHelper::find_project_pda(
                &fixture.program_id_sol_mind,
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let minter_config = AccountHelper::get_minter_config(
                &fixture.svm,
                &fixture.program_id_token_manager,
                &project_config_pda,
                &MINTER_NAME,
            );
            let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());
            let protocol_final_balance = utils::get_lamports(&fixture.svm, &protocol_config_pda);

            assert_eq!(minter_config.collection, None);
            assert_eq!(minter_config.mints_counter, 1);

            assert_eq!(asset.base.name.to_string(), ASSET_NAME.to_string());
            assert_eq!(asset.base.uri.to_string(), ASSET_URI.to_string());
            assert_eq!(
                asset.base.owner.to_string(),
                asset_owner.pubkey().to_string()
            );
            assert_eq!(
                protocol_final_balance,
                protocol_initial_balance + protocol_config.fees.mint_asset.amount
            );
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_mint_asset_with_collection() {
    let collection = Keypair::new();

    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_minter_config(PROJECT_1_ID, Some(&collection));

    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let result = Instructions::mint_asset(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME,
        Some(ASSET_NAME.to_string()),
        Some(ASSET_URI.to_string()),
        None,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        asset_owner.pubkey(),
        mint.pubkey(),
        fixture.project_authority_1.pubkey(),
        Some(collection.pubkey()),
        &[
            &fixture.payer.insecure_clone(),
            &asset_owner.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
            &mint.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let (project_config_pda, _) = AccountHelper::find_project_pda(
                &fixture.program_id_sol_mind,
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let minter_config = AccountHelper::get_minter_config(
                &fixture.svm,
                &fixture.program_id_token_manager,
                &project_config_pda,
                &MINTER_NAME,
            );
            let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());

            assert_eq!(minter_config.collection.unwrap(), collection.pubkey());
            assert_eq!(
                asset.base.owner.to_string(),
                asset_owner.pubkey().to_string()
            );
            assert_eq!(asset.base.name.to_string(), ASSET_NAME.to_string());
            assert_eq!(asset.base.uri.to_string(), ASSET_URI.to_string());

            match &asset.base.update_authority {
                mpl_core::types::UpdateAuthority::Collection(pubkey) => {
                    assert_eq!(pubkey.to_string(), collection.pubkey().to_string());
                }
                _ => panic!(
                    "Expected update_authority to be the collection, got {:?}",
                    asset.base.update_authority
                ),
            }
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_mint_asset_exceeds_max_supply() {
    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_minter_config(PROJECT_1_ID, None);

    for i in 0..MAX_SUPPLY {
        let asset_owner = Keypair::new();
        let mint = Keypair::new();

        let result = Instructions::mint_asset(
            &mut fixture.svm,
            &fixture.program_id_token_manager,
            &fixture.program_id_sol_mind,
            MINTER_NAME,
            Some(format!("{} #{}", ASSET_NAME, i)),
            Some(ASSET_URI.to_string()),
            None,
            PROJECT_1_ID,
            fixture.project_owner.pubkey(),
            fixture.payer.pubkey(),
            asset_owner.pubkey(),
            mint.pubkey(),
            fixture.project_authority_1.pubkey(),
            None,
            &[
                &fixture.payer.insecure_clone(),
                &asset_owner.insecure_clone(),
                &fixture.project_authority_1.insecure_clone(),
                &mint.insecure_clone(),
            ],
        );

        assert!(result.is_ok(), "Minting asset {} should succeed", i);
    }

    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let result = Instructions::mint_asset(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME,
        Some(ASSET_NAME.to_string()),
        Some(ASSET_URI.to_string()),
        None,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        asset_owner.pubkey(),
        mint.pubkey(),
        fixture.project_authority_1.pubkey(),
        None,
        &[
            &fixture.payer.insecure_clone(),
            &asset_owner.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
            &mint.insecure_clone(),
        ],
    );

    let (project_config_pda, _) = AccountHelper::find_project_pda(
        &fixture.program_id_sol_mind,
        &fixture.project_owner.pubkey(),
        PROJECT_1_ID,
    );
    let minter_config = AccountHelper::get_minter_config(
        &fixture.svm,
        &fixture.program_id_token_manager,
        &project_config_pda,
        &MINTER_NAME,
    );

    assert!(result.is_err(), "Minting beyond max_supply should fail");

    if let Err(e) = result {
        let error_string = format!("{:?}", e);
        assert!(
            error_string.contains("MaxSupplyReached"),
            "Error should indicate max supply reached, got: {:?}",
            e
        );
    }

    assert_eq!(minter_config.mints_counter, MAX_SUPPLY);
}

#[test]
fn test_mint_asset_with_unauthorized_authority() {
    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_minter_config(PROJECT_1_ID, None);

    let unauthorized_authority = Keypair::new();
    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    fixture
        .svm
        .airdrop(&unauthorized_authority.pubkey(), 1 * LAMPORTS_PER_SOL)
        .expect("Failed to fund unauthorized authority");

    let result = Instructions::mint_asset(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME,
        Some(ASSET_NAME.to_string()),
        Some(ASSET_URI.to_string()),
        None,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        asset_owner.pubkey(),
        mint.pubkey(),
        unauthorized_authority.pubkey(),
        None,
        &[
            &fixture.payer.insecure_clone(),
            &asset_owner.insecure_clone(),
            &unauthorized_authority.insecure_clone(),
            &mint.insecure_clone(),
        ],
    );

    assert!(
        result.is_err(),
        "Transaction should have failed with unauthorized authority"
    );

    if let Err(e) = result {
        let error_string = format!("{:?}", e);
        assert!(
            error_string.contains("Unauthorized"),
            "Error should indicate unauthorized access, got: {:?}",
            e
        );
    }
}

#[test]
fn test_create_duplicate_minter_config_name() {
    let mut fixture = TestFixture::new()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID);

    let result1 = Instructions::create_minter_config(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        None,
        None,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        fixture.project_authority_1.pubkey(),
        None,
        &[
            &fixture.payer.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
        ],
    );

    assert!(
        result1.is_ok(),
        "First minter_config creation should succeed"
    );

    let result2 = Instructions::create_minter_config(
        &mut fixture.svm,
        &fixture.program_id_token_manager,
        &fixture.program_id_sol_mind,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        None,
        None,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        fixture.project_authority_1.pubkey(),
        None,
        &[
            &fixture.payer.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
        ],
    );

    assert!(
        result2.is_err(),
        "Creating duplicate minter_config with same name should fail"
    );

    if let Err(e) = result2 {
        let error_string = format!("{:?}", e);
        assert!(
            error_string.contains("AlreadyProcessed"),
            "Error should indicate duplicate creation failed, got: {:?}",
            e
        );
    }
}
