use anchor_lang::AnchorSerialize;
use mpl_core::types::{Creator, Plugin, PluginAuthority, PluginAuthorityPair, Royalties};
use solana_program::pubkey::Pubkey as ProgramPubkey;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{Keypair, Signer},
};

use crate::setup::constants::*;
use crate::setup::TestFixture;

#[test]
fn test_initialize_project() {
    let mut fixture = TestFixture::new();
    let instructions = fixture.instructions();

    let project_name = "Test project".to_string();
    let project_description = "Project description".to_string();

    let result = instructions.initialize_project(
        &mut fixture.svm,
        PROJECT_1_ID,
        project_name,
        project_description,
        fixture.owner.pubkey(),
        fixture.treasury,
        vec![
            fixture.project_authority_1.pubkey(),
            fixture.project_authority_2.pubkey(),
        ],
        fixture.owner.pubkey(),
        &[&fixture.owner.insecure_clone()],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let account_helper = fixture.account_helper();
            let project_config =
                account_helper.get_project_config(fixture.owner.pubkey(), PROJECT_1_ID);

            assert_eq!(project_config.owner, fixture.owner.pubkey());
            assert_eq!(project_config.treasury, fixture.treasury);
            assert_eq!(project_config.minter_config_counter, 0);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_create_minter_config_without_collection() {
    let mut fixture = TestFixture::new().with_initialize_project(PROJECT_1_ID);

    let instructions = fixture.instructions();

    let result = instructions.create_minter_config(
        &mut fixture.svm,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        None,
        None,
        PROJECT_1_ID,
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

            let account_helper = fixture.account_helper();
            let project_config =
                account_helper.get_project_config(fixture.owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                account_helper.get_minter_config(fixture.owner.pubkey(), PROJECT_1_ID);

            assert_eq!(project_config.minter_config_counter, 1);
            assert_eq!(minter_config.collection, None);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_create_minter_config_with_unauthorized_authority() {
    let mut fixture = TestFixture::new().with_initialize_project(PROJECT_1_ID);

    let unauthorized_authority = Keypair::new();

    fixture
        .svm
        .airdrop(&unauthorized_authority.pubkey(), 1 * LAMPORTS_PER_SOL)
        .expect("Failed to fund unauthorized authority");

    let instructions = fixture.instructions();

    let result = instructions.create_minter_config(
        &mut fixture.svm,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        None,
        None,
        PROJECT_1_ID,
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
    }
}

#[test]
fn test_create_minter_config_with_collection() {
    let mut fixture = TestFixture::new()
        .with_initialize_project(PROJECT_1_ID)
        .with_metaplex_core_program();

    let collection = Keypair::new();
    let instructions = fixture.instructions();

    let result = instructions.create_minter_config(
        &mut fixture.svm,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        None,
        Some(COLLECTION_URI.to_string()),
        PROJECT_1_ID,
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

            let account_helper = fixture.account_helper();
            let project_config =
                account_helper.get_project_config(fixture.owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                account_helper.get_minter_config(fixture.owner.pubkey(), PROJECT_1_ID);

            assert_eq!(project_config.minter_config_counter, 1);
            assert_eq!(minter_config.collection.unwrap(), collection.pubkey());
        }
        Err(e) => {
            println!("Transaction error: {:?}", e);
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_create_minter_config_with_collection_with_plugins() {
    let mut fixture = TestFixture::new()
        .with_initialize_project(PROJECT_1_ID)
        .with_metaplex_core_program();

    let collection = Keypair::new();
    let instructions = fixture.instructions();

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

    let result = instructions.create_minter_config(
        &mut fixture.svm,
        MINTER_NAME.to_string(),
        MINT_PRICE,
        MAX_SUPPLY,
        None,
        plugins,
        Some(COLLECTION_URI.to_string()),
        PROJECT_1_ID,
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

            let account_helper = fixture.account_helper();
            let project_config =
                account_helper.get_project_config(fixture.owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                account_helper.get_minter_config(fixture.owner.pubkey(), PROJECT_1_ID);
            let asset = account_helper.get_collection(collection.pubkey());

            println!("plugins: {:?}", asset.plugin_list);

            assert_eq!(project_config.minter_config_counter, 1);
            assert_eq!(minter_config.collection.unwrap(), collection.pubkey());
            assert_ne!(asset.plugin_list.royalties, None);
        }
        Err(e) => {
            println!("Transaction error: {:?}", e);
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_mint_asset_without_assets_config_and_collection() {
    let mut fixture = TestFixture::new()
        .with_initialize_project(PROJECT_1_ID)
        .with_minter_config(PROJECT_1_ID, None)
        .with_metaplex_core_program();

    let asset_owner = Keypair::new();
    let mint = Keypair::new();
    let instructions = fixture.instructions();

    let result = instructions.mint_asset(
        &mut fixture.svm,
        Some(ASSET_NAME.to_string()),
        Some(ASSET_URI.to_string()),
        None,
        PROJECT_1_ID,
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

            let account_helper = fixture.account_helper();
            let project_config =
                account_helper.get_project_config(fixture.owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                account_helper.get_minter_config(fixture.owner.pubkey(), PROJECT_1_ID);

            assert_eq!(project_config.minter_config_counter, 1);
            assert_eq!(minter_config.collection, None);
            assert_eq!(minter_config.mints_counter, 1);
        }
        Err(e) => {
            println!("Transaction error: {:?}", e);
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_mint_asset_with_collection() {
    let collection = Keypair::new();

    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_project(PROJECT_1_ID)
        .with_minter_config(PROJECT_1_ID, Some(&collection));

    let asset_owner = Keypair::new();
    let mint = Keypair::new();
    let instructions = fixture.instructions();

    let result = instructions.mint_asset(
        &mut fixture.svm,
        Some(ASSET_NAME.to_string()),
        Some(ASSET_URI.to_string()),
        None,
        PROJECT_1_ID,
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

            let account_helper = fixture.account_helper();
            let minter_config =
                account_helper.get_minter_config(fixture.owner.pubkey(), PROJECT_1_ID);
            let asset = account_helper.get_asset(mint.pubkey());

            assert_eq!(minter_config.collection.unwrap(), collection.pubkey());
            assert_eq!(minter_config.mints_counter, 1);
            assert_eq!(
                asset.base.owner.to_string(),
                asset_owner.pubkey().to_string()
            );
            assert_eq!(asset.base.name.to_string(), ASSET_NAME.to_string());
            assert_eq!(asset.base.uri.to_string(), ASSET_URI.to_string());

            match &asset.base.update_authority {
                mpl_core::types::UpdateAuthority::Collection(pubkey) => {
                    println!("collection: {:?}", collection.pubkey());
                    println!("update: {:?}", pubkey.to_string());
                    assert_eq!(pubkey.to_string(), collection.pubkey().to_string());
                }
                _ => panic!(
                    "Expected update_authority to be the collection, got {:?}",
                    asset.base.update_authority
                ),
            }
        }
        Err(e) => {
            println!("Transaction error: {:?}", e);
            panic!("Transaction failed: {:?}", e);
        }
    }
}
