use litesvm::{types::TransactionResult, LiteSVM};
use sol_mind_protocol_client::token_manager::{
    instructions::{CreateMinterConfigBuilder, MintAssetBuilder},
    types::AssetsConfig,
};
use sol_mind_protocol_client::{
    instructions::{
        CreateProjectBuilder, InitializeProtocolBuilder, TransferProjectFeesBuilder,
        TransferProtocolFeesBuilder, UpdateFeesBuilder, UpdateSingleFeeBuilder,
    },
    types::{Fee, FeesStructure, Operation},
};
use solana_pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID;

use super::accounts::AccountHelper;

pub struct Instructions;

impl Instructions {
    // Sol-mind-protocol instructions
    pub fn initialize_protocol(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        admins: Vec<Pubkey>,
        whitelist_transfer_addrs: Vec<Pubkey>,
        fees: FeesStructure,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = InitializeProtocolBuilder::new()
            .payer(payer)
            .protocol_config(protocol_config_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .admins(admins)
            .whitelist_transfer_addrs(whitelist_transfer_addrs)
            .fees(fees)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn create_project(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        project_id: u64,
        name: String,
        description: String,
        owner: Pubkey,
        authorities: Vec<Pubkey>,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (project_config_pda, _) =
            AccountHelper::find_project_pda(program_id, &owner, project_id);
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);
        let (treasury_pda, _) = AccountHelper::find_treasury_pda(program_id, &project_config_pda);

        let instruction = CreateProjectBuilder::new()
            .owner(owner)
            .project_config(project_config_pda)
            .protocol_config(protocol_config_pda)
            .treasury(treasury_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .project_id(project_id)
            .name(name)
            .description(description)
            .authorities(authorities)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn update_fees(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        fees: FeesStructure,
        admin: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = UpdateFeesBuilder::new()
            .admin(admin)
            .protocol_config(protocol_config_pda)
            .fees(fees)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn update_single_fee(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        operation: Operation,
        fee: Fee,
        admin: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = UpdateSingleFeeBuilder::new()
            .admin(admin)
            .protocol_config(protocol_config_pda)
            .operation(operation)
            .fee(fee)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn transfer_project_fees(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        amount: u64,
        owner: Pubkey,
        to: Pubkey,
        project_id: u64,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (project_config_pda, _) =
            AccountHelper::find_project_pda(program_id, &owner, project_id);
        let (treasury_pda, _) = AccountHelper::find_treasury_pda(program_id, &project_config_pda);

        let instruction = TransferProjectFeesBuilder::new()
            .owner(owner)
            .to(to)
            .project_config(project_config_pda)
            .treasury(treasury_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .amount(amount)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn protocol_fees_transfer(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        amount: u64,
        admin: Pubkey,
        to: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = TransferProtocolFeesBuilder::new()
            .admin(admin)
            .to(to)
            .protocol_config(protocol_config_pda)
            .amount(amount)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn create_minter_config(
        svm: &mut LiteSVM,
        token_manager_program_id: &Pubkey,
        sol_mind_protocol_program_id: &Pubkey,
        name: String,
        mint_price: u64,
        max_supply: u64,
        assets_config: Option<AssetsConfig>,
        plugins: Option<Vec<Vec<u8>>>,
        uri: Option<String>,
        project_id: u64,
        owner: Pubkey,
        payer: Pubkey,
        authority: Pubkey,
        collection: Option<Pubkey>,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) =
            AccountHelper::find_protocol_config_pda(sol_mind_protocol_program_id);
        let (project_config_pda, _) =
            AccountHelper::find_project_pda(sol_mind_protocol_program_id, &owner, project_id);
        let (minter_config_pda, _) =
            AccountHelper::find_minter_config_pda(token_manager_program_id, project_id, &name);

        let mut builder = CreateMinterConfigBuilder::new();

        builder
            .name(name)
            .mint_price(mint_price)
            .max_supply(max_supply)
            .payer(payer)
            .authority(authority)
            .collection(collection)
            .minter_config(minter_config_pda)
            .project_config(project_config_pda)
            .protocol_config(protocol_config_pda);

        if let Some(assets_config) = assets_config {
            builder.assets_config(assets_config);
        }
        if let Some(plugins) = plugins {
            builder.plugins(plugins);
        }
        if let Some(uri) = uri {
            builder.uri(uri);
        }

        utils::send_transaction(svm, &[builder.instruction()], &payer, signing_keypairs)
    }

    pub fn mint_asset(
        svm: &mut LiteSVM,
        token_manager_program_id: &Pubkey,
        sol_mind_protocol_program_id: &Pubkey,
        minter_config_name: &str,
        name: Option<String>,
        uri: Option<String>,
        plugins: Option<Vec<Vec<u8>>>,
        project_id: u64,
        owner: Pubkey,
        payer: Pubkey,
        asset_owner: Pubkey,
        mint: Pubkey,
        authority: Pubkey,
        collection: Option<Pubkey>,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) =
            AccountHelper::find_protocol_config_pda(sol_mind_protocol_program_id);
        let (project_config_pda, _) =
            AccountHelper::find_project_pda(sol_mind_protocol_program_id, &owner, project_id);
        let (minter_config_pda, _) = AccountHelper::find_minter_config_pda(
            token_manager_program_id,
            project_id,
            minter_config_name,
        );

        let mut builder = MintAssetBuilder::new();

        builder
            .payer(payer)
            .owner(asset_owner)
            .mint(mint)
            .authority(authority)
            .collection(collection)
            .minter_config(minter_config_pda)
            .project_config(project_config_pda)
            .protocol_config(protocol_config_pda);

        if let Some(name) = name {
            builder.name(name);
        }
        if let Some(uri) = uri {
            builder.uri(uri);
        }
        if let Some(plugins) = plugins {
            builder.plugins(plugins);
        }

        utils::send_transaction(svm, &[builder.instruction()], &payer, signing_keypairs)
    }
}
