mod setup;

use anchor_lang::AnchorSerialize;
use mpl_core::types::{Creator, Plugin, PluginAuthority, PluginAuthorityPair, Royalties};
use solana_program::pubkey::Pubkey as ProgramPubkey;
use solana_sdk::{
    clock::Clock,
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

    let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda();
    let (protocol_treasury_pda, _) = AccountHelper::find_treasury_pda(&protocol_config_pda);
    let protocol_treasury_initial_balance = utils::get_lamports(&fixture.svm, &protocol_treasury_pda);

    let result = Instructions::create_minter_config(
        &mut fixture.svm,
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

            let protocol_config = AccountHelper::get_protocol_config(&fixture.svm);
            let (project_config_pda, _) =
                AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                AccountHelper::get_minter_config(&fixture.svm, &project_config_pda, &MINTER_NAME);
            let protocol_treasury_final_balance =
                utils::get_lamports(&fixture.svm, &protocol_treasury_pda);

            assert_eq!(minter_config.name, MINTER_NAME);
            assert_eq!(minter_config.mint_price, MINT_PRICE);
            assert_eq!(minter_config.max_supply, MAX_SUPPLY);
            assert_eq!(minter_config.mints_counter, 0);
            assert_eq!(minter_config.collection, None);
            assert_eq!(minter_config.assets_config, None);
            assert_eq!(
                protocol_treasury_final_balance,
                protocol_treasury_initial_balance + protocol_config.fees.create_minter_config.amount
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

            let (project_config_pda, _) =
                AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                AccountHelper::get_minter_config(&fixture.svm, &project_config_pda, &MINTER_NAME);

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
                &fixture.project_owner.pubkey(),
                PROJECT_1_ID,
            );
            let (project_config_pda, _) =
                AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                AccountHelper::get_minter_config(&fixture.svm, &project_config_pda, &MINTER_NAME);
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
        .with_create_minter_config(PROJECT_1_ID, None);

    let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda();
    let (protocol_treasury_pda, _) = AccountHelper::find_treasury_pda(&protocol_config_pda);
    let protocol_treasury_initial_balance = utils::get_lamports(&fixture.svm, &protocol_treasury_pda);

    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let result = Instructions::mint_asset(
        &mut fixture.svm,
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

            let protocol_config = AccountHelper::get_protocol_config(&fixture.svm);
            let (project_config_pda, _) =
                AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                AccountHelper::get_minter_config(&fixture.svm, &project_config_pda, &MINTER_NAME);
            let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());
            let protocol_treasury_final_balance =
                utils::get_lamports(&fixture.svm, &protocol_treasury_pda);

            assert_eq!(minter_config.collection, None);
            assert_eq!(minter_config.mints_counter, 1);

            assert_eq!(asset.base.name.to_string(), ASSET_NAME.to_string());
            assert_eq!(asset.base.uri.to_string(), ASSET_URI.to_string());
            assert_eq!(
                asset.base.owner.to_string(),
                asset_owner.pubkey().to_string()
            );
            assert_eq!(
                protocol_treasury_final_balance,
                protocol_treasury_initial_balance + protocol_config.fees.mint_asset.amount
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
        .with_create_minter_config(PROJECT_1_ID, Some(&collection));

    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let result = Instructions::mint_asset(
        &mut fixture.svm,
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

            let (project_config_pda, _) =
                AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID);
            let minter_config =
                AccountHelper::get_minter_config(&fixture.svm, &project_config_pda, &MINTER_NAME);
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
        .with_create_minter_config(PROJECT_1_ID, None);

    for i in 0..MAX_SUPPLY {
        let asset_owner = Keypair::new();
        let mint = Keypair::new();

        let result = Instructions::mint_asset(
            &mut fixture.svm,
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

    let (project_config_pda, _) =
        AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID);
    let minter_config =
        AccountHelper::get_minter_config(&fixture.svm, &project_config_pda, &MINTER_NAME);

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
        .with_create_minter_config(PROJECT_1_ID, None);

    let unauthorized_authority = Keypair::new();
    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    fixture
        .svm
        .airdrop(&unauthorized_authority.pubkey(), 1 * LAMPORTS_PER_SOL)
        .expect("Failed to fund unauthorized authority");

    let result = Instructions::mint_asset(
        &mut fixture.svm,
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
        .with_initialize_project(PROJECT_1_ID)
        .with_create_minter_config(PROJECT_1_ID, None);

    let result2 = Instructions::create_minter_config(
        &mut fixture.svm,
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
            error_string.contains("already in use"),
            "Error should indicate duplicate creation failed, got: {:?}",
            e
        );
    }
}

#[test]
fn test_create_trade_hub() {
    let mut fixture = TestFixture::new()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID);

    let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;
    let (protocol_treasury_pda, _) = AccountHelper::find_treasury_pda(&protocol_config_pda);
    let protocol_treasury_initial_balance = utils::get_lamports(&fixture.svm, &protocol_treasury_pda);

    let result = Instructions::create_trade_hub(
        &mut fixture.svm,
        TRADE_HUB_NAME.to_string(),
        TRADE_HUB_FEE_BPS,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        fixture.project_authority_1.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.project_authority_1.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let protocol_config = AccountHelper::get_protocol_config(&fixture.svm);
            let project_config_pda =
                AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID).0;
            let trade_hub =
                AccountHelper::get_trade_hub(&fixture.svm, TRADE_HUB_NAME, &project_config_pda);
            let protocol_treasury_final_balance =
                utils::get_lamports(&fixture.svm, &protocol_treasury_pda);

            let trade_hub_pda =
                AccountHelper::find_trade_hub_pda(TRADE_HUB_NAME, &project_config_pda).0;
            assert!(
                fixture.svm.get_account(&trade_hub_pda).is_some(),
                "Trade hub account should exist"
            );
            assert_eq!(trade_hub.name, TRADE_HUB_NAME);
            assert_eq!(trade_hub.fee_bps, TRADE_HUB_FEE_BPS);
            assert_eq!(trade_hub.project, project_config_pda);
            assert_eq!(
                protocol_treasury_final_balance,
                protocol_treasury_initial_balance + protocol_config.fees.create_trade_hub.amount
            );
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_create_trade_hub_with_unauthorized_authority() {
    let mut fixture = TestFixture::new()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID);

    let unauthorized_authority = Keypair::new();

    fixture
        .svm
        .airdrop(&unauthorized_authority.pubkey(), 1 * LAMPORTS_PER_SOL)
        .expect("Failed to fund unauthorized authority");

    let result = Instructions::create_trade_hub(
        &mut fixture.svm,
        TRADE_HUB_NAME.to_string(),
        TRADE_HUB_FEE_BPS,
        PROJECT_1_ID,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        unauthorized_authority.pubkey(),
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
        let error_string = format!("{:?}", e);
        assert!(
            error_string.contains("Unauthorized"),
            "Error should indicate unauthorized access, got: {:?}",
            e
        );
    }
}

#[test]
fn test_list_asset() {
    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_create_minter_config(PROJECT_1_ID, None)
        .with_create_trade_hub(PROJECT_1_ID)
        .with_minted_asset(PROJECT_1_ID, &asset_owner, &mint, None);

    let mut clock: Clock = fixture.svm.get_sysvar();
    clock.unix_timestamp = 1000;
    // fixture.svm.warp_to_slot(clock.slot + 100); // Doesn't increase unix_timestamp
    fixture.svm.set_sysvar(&clock);

    let project_config_pda =
        AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID).0;

    let result = Instructions::list_asset(
        &mut fixture.svm,
        LISTING_PRICE,
        fixture.payer.pubkey(),
        &asset_owner.pubkey(),
        &mint.pubkey(),
        TRADE_HUB_NAME,
        &project_config_pda,
        None,
        &[
            &fixture.payer.insecure_clone(),
            &asset_owner.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let trade_hub_pda =
                AccountHelper::find_trade_hub_pda(TRADE_HUB_NAME, &project_config_pda).0;
            let listing =
                AccountHelper::get_listing(&fixture.svm, &mint.pubkey(), &trade_hub_pda).unwrap();
            let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());

            assert_eq!(listing.owner, asset_owner.pubkey());
            assert_eq!(listing.asset, mint.pubkey());
            assert_eq!(listing.price, LISTING_PRICE);
            assert!(listing.created_at > 0);
            let freeze_plugin = asset
                .plugin_list
                .freeze_delegate
                .expect("Freeze delegate plugin should exist");
            assert_eq!(freeze_plugin.freeze_delegate.frozen, true);

            let transfer_authority = asset
                .plugin_list
                .transfer_delegate
                .and_then(|plugin| plugin.base.authority.address)
                .expect("Transfer delegate plugin should exist with trade hub authority");
            assert_eq!(transfer_authority.to_string(), trade_hub_pda.to_string());
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_list_asset_with_collection() {
    let collection = Keypair::new();
    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_create_minter_config(PROJECT_1_ID, Some(&collection))
        .with_create_trade_hub(PROJECT_1_ID)
        .with_minted_asset(PROJECT_1_ID, &asset_owner, &mint, Some(collection.pubkey()));

    let (project_config_pda, _) =
        AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID);

    let mut clock: Clock = fixture.svm.get_sysvar();
    clock.unix_timestamp = 1000;
    fixture.svm.set_sysvar(&clock);

    let result = Instructions::list_asset(
        &mut fixture.svm,
        LISTING_PRICE,
        fixture.payer.pubkey(),
        &asset_owner.pubkey(),
        &mint.pubkey(),
        TRADE_HUB_NAME,
        &project_config_pda,
        Some(collection.pubkey()),
        &[
            &fixture.payer.insecure_clone(),
            &asset_owner.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let trade_hub_pda =
                AccountHelper::find_trade_hub_pda(TRADE_HUB_NAME, &project_config_pda).0;
            let listing =
                AccountHelper::get_listing(&fixture.svm, &mint.pubkey(), &trade_hub_pda).unwrap();

            assert_eq!(listing.owner, asset_owner.pubkey());
            assert_eq!(listing.asset, mint.pubkey());
            assert_eq!(listing.price, LISTING_PRICE);
            let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());
            assert!(asset.plugin_list.transfer_delegate.is_some());
            assert!(asset.plugin_list.freeze_delegate.is_some());

            assert!(listing.created_at > 0);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_list_asset_by_non_owner() {
    let asset_owner = Keypair::new();
    let non_owner = Keypair::new();
    let mint = Keypair::new();

    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_create_minter_config(PROJECT_1_ID, None)
        .with_create_trade_hub(PROJECT_1_ID)
        .with_minted_asset(PROJECT_1_ID, &asset_owner, &mint, None);

    fixture
        .svm
        .airdrop(&non_owner.pubkey(), 1 * LAMPORTS_PER_SOL)
        .expect("Failed to fund non-owner");

    let project_config_pda =
        AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID).0;

    let result = Instructions::list_asset(
        &mut fixture.svm,
        LISTING_PRICE,
        fixture.payer.pubkey(),
        &non_owner.pubkey(),
        &mint.pubkey(),
        TRADE_HUB_NAME,
        &project_config_pda,
        None,
        &[&fixture.payer.insecure_clone(), &non_owner.insecure_clone()],
    );

    assert!(
        result.is_err(),
        "Transaction should have failed with non-owner"
    );

    if let Err(e) = result {
        let error_string = format!("{:?}", e);
        assert!(
            error_string.contains("Neither the asset or any plugins have approved this operation"),
            "Error should indicate unauthorized access, got: {:?}",
            e
        );
    }
}

#[test]
fn test_purchase_asset() {
    let asset_owner = Keypair::new();
    let buyer = Keypair::new();
    let mint = Keypair::new();

    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_create_minter_config(PROJECT_1_ID, None)
        .with_create_trade_hub(PROJECT_1_ID)
        .with_minted_asset(PROJECT_1_ID, &asset_owner, &mint, None)
        .with_list_asset(PROJECT_1_ID, mint.pubkey(), &asset_owner, None);

    fixture
        .svm
        .airdrop(&asset_owner.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to fund owner");

    fixture
        .svm
        .airdrop(&buyer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to fund buyer");

    let project_config_pda =
        AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID).0;
    let protocol_config = AccountHelper::get_protocol_config(&fixture.svm);
    let protocol_treasury_pda = AccountHelper::find_treasury_pda(&AccountHelper::find_protocol_config_pda().0).0;
    let protocol_treasury_initial_balance = utils::get_lamports(&fixture.svm, &protocol_treasury_pda);
    let treasury_pda = AccountHelper::find_treasury_pda(&project_config_pda).0;
    let trade_hub_pda =
        AccountHelper::find_trade_hub_pda(TRADE_HUB_NAME, &project_config_pda).0;
    let listing_pda =
        AccountHelper::find_listing_pda(&mint.pubkey(), &trade_hub_pda).0;

    let treasury_initial_balance = utils::get_lamports(&fixture.svm, &treasury_pda);
    let owner_initial_balance = utils::get_lamports(&fixture.svm, &asset_owner.pubkey());
    let listing_balance = utils::get_lamports(&fixture.svm, &listing_pda);

    let result = Instructions::purchase_asset(
        &mut fixture.svm,
        buyer.pubkey(),
        &asset_owner.pubkey(),
        &mint.pubkey(),
        TRADE_HUB_NAME,
        &project_config_pda,
        None,
        LISTING_PRICE,
        &[&buyer.insecure_clone()],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let listing = AccountHelper::get_listing(&fixture.svm, &mint.pubkey(), &trade_hub_pda);
            let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());
            let protocol_treasury_final_balance =
                utils::get_lamports(&fixture.svm, &AccountHelper::find_treasury_pda(&AccountHelper::find_protocol_config_pda().0).0);
            let treasury_final_balance = utils::get_lamports(&fixture.svm, &treasury_pda);
            let owner_final_balance = utils::get_lamports(&fixture.svm, &asset_owner.pubkey());

            assert_eq!(
                asset.base.owner.to_string(),
                buyer.pubkey().to_string(),
                "Asset should be transferred to buyer"
            );

            println!("fee type: {:?}", protocol_config.fees.trade_nft.fee_type);

            let protocol_fee = match protocol_config.fees.trade_nft.fee_type {
                sol_mind_protocol_client::types::FeeType::Percentage => LISTING_PRICE
                    .checked_mul(protocol_config.fees.trade_nft.amount)
                    .unwrap()
                    .checked_div(10_000)
                    .unwrap(),
                _ => panic!("Wrong fee type, should be Percentage"),
            };
            let trade_hub =
                AccountHelper::get_trade_hub(&fixture.svm, TRADE_HUB_NAME, &project_config_pda);
            let trade_hub_fee = LISTING_PRICE
                .checked_mul(trade_hub.fee_bps)
                .unwrap()
                .checked_div(10_000)
                .unwrap();
            let seller_amount = LISTING_PRICE
                .checked_sub(protocol_fee)
                .unwrap()
                .checked_sub(trade_hub_fee)
                .unwrap();

            assert_eq!(
                protocol_treasury_final_balance,
                protocol_treasury_initial_balance + protocol_fee,
                "Protocol should receive fee"
            );
            assert_eq!(
                treasury_final_balance,
                treasury_initial_balance + trade_hub_fee,
                "Treasury should receive trade hub fee"
            );
            assert_eq!(
                owner_final_balance,
                owner_initial_balance + seller_amount + listing_balance,
                "Owner should receive seller amount and the listing rent"
            );
            let freeze_plugin = asset
                .plugin_list
                .freeze_delegate
                .expect("Freeze delegate plugin should exist");
            assert_eq!(freeze_plugin.freeze_delegate.frozen, false);

            let transfer_plugin = asset
                .plugin_list
                .transfer_delegate
                .expect("Transfer delegate plugin should exist");
            assert_eq!(transfer_plugin.base.authority.address, None);
            assert!(listing.is_none())
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_purchase_asset_with_collection() {
    let collection = Keypair::new();
    let asset_owner = Keypair::new();
    let buyer = Keypair::new();
    let mint = Keypair::new();

    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_create_minter_config(PROJECT_1_ID, Some(&collection))
        .with_create_trade_hub(PROJECT_1_ID)
        .with_minted_asset(PROJECT_1_ID, &asset_owner, &mint, Some(collection.pubkey()))
        .with_list_asset(
            PROJECT_1_ID,
            mint.pubkey(),
            &asset_owner,
            Some(collection.pubkey()),
        );

    fixture
        .svm
        .airdrop(&buyer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to fund buyer");

    let project_config_pda =
        AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID).0;

    let result = Instructions::purchase_asset(
        &mut fixture.svm,
        buyer.pubkey(),
        &asset_owner.pubkey(),
        &mint.pubkey(),
        TRADE_HUB_NAME,
        &project_config_pda,
        Some(collection.pubkey()),
        LISTING_PRICE,
        &[&buyer.insecure_clone()],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());

            assert_eq!(
                asset.base.owner.to_string(),
                buyer.pubkey().to_string(),
                "Asset should be transferred to buyer"
            );
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_delist_asset() {
    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let mut fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_create_minter_config(PROJECT_1_ID, None)
        .with_create_trade_hub(PROJECT_1_ID)
        .with_minted_asset(PROJECT_1_ID, &asset_owner, &mint, None)
        .with_list_asset(PROJECT_1_ID, mint.pubkey(), &asset_owner, None);

    let project_config_pda =
        AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID).0;

    let result = Instructions::delist_asset(
        &mut fixture.svm,
        &fixture.payer.pubkey(),
        &asset_owner.pubkey(),
        &mint.pubkey(),
        TRADE_HUB_NAME,
        &project_config_pda,
        None,
        &[
            &fixture.payer.insecure_clone(),
            &asset_owner.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let trade_hub_pda =
                AccountHelper::find_trade_hub_pda(TRADE_HUB_NAME, &project_config_pda).0;
            let listing = AccountHelper::get_listing(&fixture.svm, &mint.pubkey(), &trade_hub_pda);
            let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());

            assert!(asset.plugin_list.freeze_delegate.is_none());
            assert!(asset.plugin_list.transfer_delegate.is_none());
            assert!(listing.is_none());
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_deslist_asset_and_than_list_again() {
    let asset_owner = Keypair::new();
    let mint = Keypair::new();

    let fixture = TestFixture::new()
        .with_metaplex_core_program()
        .with_initialize_protocol()
        .with_initialize_project(PROJECT_1_ID)
        .with_create_minter_config(PROJECT_1_ID, None)
        .with_create_trade_hub(PROJECT_1_ID)
        .with_minted_asset(PROJECT_1_ID, &asset_owner, &mint, None)
        .with_list_asset(PROJECT_1_ID, mint.pubkey(), &asset_owner, None)
        .with_delist_asset(PROJECT_1_ID, mint.pubkey(), &asset_owner, None)
        .with_list_asset(PROJECT_1_ID, mint.pubkey(), &asset_owner, None);

    let project_config_pda =
        AccountHelper::find_project_pda(&fixture.project_owner.pubkey(), PROJECT_1_ID).0;
    let trade_hub_pda =
        AccountHelper::find_trade_hub_pda(TRADE_HUB_NAME, &project_config_pda).0;
        
    let listing = AccountHelper::get_listing(&fixture.svm, &mint.pubkey(), &trade_hub_pda);
    let asset = MplUtils::get_asset(&fixture.svm, &mint.pubkey());

    assert!(asset.plugin_list.freeze_delegate.is_some());
    assert!(asset.plugin_list.transfer_delegate.is_some());
    assert!(listing.is_some());
    assert_eq!(listing.unwrap().price, LISTING_PRICE);

}
