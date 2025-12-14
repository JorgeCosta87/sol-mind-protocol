use litesvm::{types::TransactionResult, LiteSVM};
use sol_mind_protocol_client::nft_operations::{
    instructions::{
        CreateMinterConfigBuilder, CreateTradeHubBuilder, DelistAssetBuilder, ListAssetBuilder,
        MintAssetBuilder, PurchaseAssetBuilder,
    },
    types::AssetsConfig,
};
use sol_mind_protocol_client::{
    dac_manager::instructions::{
        ClaimComputeNodeBuilder, CreateAgentBuilder, RegisterComputeNodeBuilder, SubmitTaskBuilder,
    },
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
    pub fn initialize_protocol(
        svm: &mut LiteSVM,
        admins: Vec<Pubkey>,
        whitelist_transfer_addrs: Vec<Pubkey>,
        fees: FeesStructure,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;

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
        project_id: u64,
        name: String,
        description: String,
        owner: Pubkey,
        authorities: Vec<Pubkey>,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;
        let project_config_pda = AccountHelper::find_project_pda(&owner, project_id).0;
        let treasury_pda = AccountHelper::find_treasury_pda(&project_config_pda).0;
        let protocol_treasury_pda = AccountHelper::find_treasury_pda(&protocol_config_pda).0;

        let instruction = CreateProjectBuilder::new()
            .owner(owner)
            .project_config(project_config_pda)
            .protocol_config(protocol_config_pda)
            .treasury(treasury_pda)
            .protocol_treasury(protocol_treasury_pda)
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
        fees: FeesStructure,
        admin: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;

        let instruction = UpdateFeesBuilder::new()
            .admin(admin)
            .protocol_config(protocol_config_pda)
            .fees(fees)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn update_single_fee(
        svm: &mut LiteSVM,
        operation: Operation,
        fee: Fee,
        admin: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;

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
        amount: u64,
        owner: Pubkey,
        to: Pubkey,
        project_id: u64,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let project_config_pda = AccountHelper::find_project_pda(&owner, project_id).0;
        let treasury_pda = AccountHelper::find_treasury_pda(&project_config_pda).0;

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
        amount: u64,
        admin: Pubkey,
        to: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;
        let treasury_pda = AccountHelper::find_treasury_pda(&protocol_config_pda).0;

        let instruction = TransferProtocolFeesBuilder::new()
            .admin(admin)
            .to(to)
            .protocol_config(protocol_config_pda)
            .treasury(treasury_pda)
            .amount(amount)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn create_minter_config(
        svm: &mut LiteSVM,
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
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;
        let project_config_pda = AccountHelper::find_project_pda(&owner, project_id).0;
        let minter_config_pda = AccountHelper::find_minter_config_pda(&project_config_pda, &name).0;
        let protocol_treasury_pda = AccountHelper::find_treasury_pda(&protocol_config_pda).0;

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
            .protocol_config(protocol_config_pda)
            .protocol_treasury(protocol_treasury_pda);

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
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;
        let project_config_pda = AccountHelper::find_project_pda(&owner, project_id).0;
        let minter_config_pda =
            AccountHelper::find_minter_config_pda(&project_config_pda, minter_config_name).0;
        let protocol_treasury_pda = AccountHelper::find_treasury_pda(&protocol_config_pda).0;

        let mut builder = MintAssetBuilder::new();

        builder
            .payer(payer)
            .owner(asset_owner)
            .mint(mint)
            .authority(authority)
            .collection(collection)
            .minter_config(minter_config_pda)
            .project_config(project_config_pda)
            .protocol_config(protocol_config_pda)
            .protocol_treasury(protocol_treasury_pda);

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

    pub fn create_trade_hub(
        svm: &mut LiteSVM,
        name: String,
        fee_bps: u64,
        project_id: u64,
        owner: Pubkey,
        payer: Pubkey,
        authority: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;
        let project_config_pda = AccountHelper::find_project_pda(&owner, project_id).0;
        let trade_hub_pda = AccountHelper::find_trade_hub_pda(&name, &project_config_pda).0;
        let protocol_treasury_pda = AccountHelper::find_treasury_pda(&protocol_config_pda).0;

        let instruction = CreateTradeHubBuilder::new()
            .payer(payer)
            .authority(authority)
            .trade_hub(trade_hub_pda)
            .project_config(project_config_pda)
            .protocol_config(protocol_config_pda)
            .protocol_treasury(protocol_treasury_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .name(name)
            .fee_bps(fee_bps)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn list_asset(
        svm: &mut LiteSVM,
        price: u64,
        payer: Pubkey,
        owner: &Pubkey,
        mint: &Pubkey,
        trade_hub_name: &str,
        project_config_pda: &Pubkey,
        collection: Option<Pubkey>,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let trade_hub_pda = AccountHelper::find_trade_hub_pda(trade_hub_name, project_config_pda).0;
        let listing_pda = AccountHelper::find_listing_pda(&mint, &trade_hub_pda).0;

        let instruction = ListAssetBuilder::new()
            .payer(payer)
            .owner(owner.clone())
            .asset(mint.clone())
            .listing(listing_pda)
            .trade_hub(trade_hub_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .collection(collection)
            .price(price)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn purchase_asset(
        svm: &mut LiteSVM,
        buyer: Pubkey,
        owner: &Pubkey,
        mint: &Pubkey,
        trade_hub_name: &str,
        project_config_pda: &Pubkey,
        collection: Option<Pubkey>,
        max_price: u64,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let trade_hub_pda = AccountHelper::find_trade_hub_pda(trade_hub_name, project_config_pda).0;
        let listing_pda = AccountHelper::find_listing_pda(&mint, &trade_hub_pda).0;
        let protocol_config_pda = AccountHelper::find_protocol_config_pda().0;
        let treasury_pda = AccountHelper::get_treasury_pda(svm, project_config_pda);
        let protocol_treasury_pda = AccountHelper::find_treasury_pda(&protocol_config_pda).0;

        let instruction = PurchaseAssetBuilder::new()
            .buyer(buyer)
            .owner(*owner)
            .asset(*mint)
            .collection(collection)
            .listing(listing_pda)
            .trade_hub(trade_hub_pda)
            .treasury(treasury_pda)
            .project_config(*project_config_pda)
            .protocol_config(protocol_config_pda)
            .protocol_treasury(protocol_treasury_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .max_price(max_price)
            .instruction();

        utils::send_transaction(svm, &[instruction], &buyer, signing_keypairs)
    }

    pub fn delist_asset(
        svm: &mut LiteSVM,
        payer: &Pubkey,
        owner: &Pubkey,
        mint: &Pubkey,
        trade_hub_name: &str,
        project_config_pda: &Pubkey,
        collection: Option<Pubkey>,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let trade_hub_pda = AccountHelper::find_trade_hub_pda(trade_hub_name, project_config_pda).0;
        let listing_pda = AccountHelper::find_listing_pda(&mint, &trade_hub_pda).0;

        let instruction = DelistAssetBuilder::new()
            .payer(*payer)
            .owner(*owner)
            .asset(*mint)
            .listing(listing_pda)
            .trade_hub(trade_hub_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .collection(collection)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn create_agent(
        svm: &mut LiteSVM,
        agent_id: u64,
        public: bool,
        compute_node_info_pda: Pubkey,
        owner: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let agent_pda = AccountHelper::find_agent_pda(&owner, agent_id).0;
        let task_data_pda = AccountHelper::find_task_data_pda(&agent_pda).0;

        let instruction = CreateAgentBuilder::new()
            .payer(payer)
            .owner(owner)
            .agent(agent_pda)
            .agent_id(agent_id)
            .public(public)
            .task_data(task_data_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .compute_node_info(compute_node_info_pda)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn submit_task(
        svm: &mut LiteSVM,
        task_data: Vec<u8>,
        agent_pda: Pubkey,
        submitter: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let task_data_pda = AccountHelper::find_task_data_pda(&agent_pda).0;

        let mut builder = SubmitTaskBuilder::new();
        builder
            .payer(payer)
            .submitter(submitter)
            .task_data(task_data_pda)
            .agent(agent_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .data(task_data);

        utils::send_transaction(svm, &[builder.instruction()], &payer, signing_keypairs)
    }

    pub fn register_compute_node(
        svm: &mut LiteSVM,
        node_pubkey: Pubkey,
        owner: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let compute_node_info_pda = AccountHelper::find_compute_node_info_pda(&node_pubkey).0;

        let instruction = RegisterComputeNodeBuilder::new()
            .payer(payer)
            .owner(owner)
            .compute_node_info(compute_node_info_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .node_pubkey(node_pubkey)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn claim_compute_node(
        svm: &mut LiteSVM,
        node_pubkey: Pubkey,
        node_info_cid: String,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let compute_node_info_pda = AccountHelper::find_compute_node_info_pda(&node_pubkey).0;

        let instruction = ClaimComputeNodeBuilder::new()
            .payer(payer)
            .compute_node(node_pubkey)
            .compute_node_info(compute_node_info_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .node_info_cid(node_info_cid)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }
}
