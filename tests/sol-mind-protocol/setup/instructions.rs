use litesvm::{types::TransactionResult, LiteSVM};
use sol_mind_protocol_client::generated::{
    instructions::{CreateMinterConfigBuilder, InitializeProjectBuilder, MintAssetBuilder},
    types::AssetsConfig,
};
use solana_pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID;

use crate::setup::accounts::AccountHelper;

pub struct Instructions {
    program_id: Pubkey,
    owner: Pubkey,
}

impl Instructions {
    pub fn new(program_id: Pubkey, owner: Pubkey) -> Self {
        Self { program_id, owner }
    }

    pub fn account_helper<'a>(&self, svm: &'a LiteSVM) -> AccountHelper<'a> {
        AccountHelper::new(svm, self.program_id)
    }

    pub fn initialize_project(
        &self,
        svm: &mut LiteSVM,
        project_id: u64,
        name: String,
        description: String,
        owner: Pubkey,
        treasury: Pubkey,
        authorities: Vec<Pubkey>,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let account_helper = self.account_helper(svm);
        let (project_config_pda, _) = account_helper.find_project_pda(owner, project_id);

        let instruction = InitializeProjectBuilder::new()
            .owner(owner)
            .project_config(project_config_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .project_id(project_id)
            .name(name)
            .description(description)
            .treasury(treasury)
            .authorities(authorities)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn create_minter_config(
        &self,
        svm: &mut LiteSVM,
        name: String,
        mint_price: u64,
        max_supply: u64,
        assets_config: Option<AssetsConfig>,
        plugins: Option<Vec<Vec<u8>>>,
        uri: Option<String>,
        project_id: u64,
        payer: Pubkey,
        authority: Pubkey,
        collection: Option<Pubkey>,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let account_helper = self.account_helper(svm);
        let (project_config_pda, _) = account_helper.find_project_pda(self.owner, project_id);
        let (minter_config_pda, _) =
            account_helper.find_next_minter_config_pda(self.owner, project_id);

        let mut builder = CreateMinterConfigBuilder::new();

        builder
            .name(name)
            .mint_price(mint_price)
            .max_supply(max_supply)
            .payer(payer)
            .authority(authority)
            .collection(collection)
            .minter_config(minter_config_pda)
            .project_config(project_config_pda);

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
        &self,
        svm: &mut LiteSVM,
        name: Option<String>,
        uri: Option<String>,
        plugins: Option<Vec<Vec<u8>>>,
        project_id: u64,
        payer: Pubkey,
        asset_owner: Pubkey,
        mint: Pubkey,
        authority: Pubkey,
        collection: Option<Pubkey>,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let account_helper = self.account_helper(svm);
        let (project_config_pda, _) = account_helper.find_project_pda(self.owner, project_id);
        let (minter_config_pda, _) = account_helper.find_minter_config_pda(self.owner, project_id);

        let mut builder = MintAssetBuilder::new();

        builder
            .payer(payer)
            .owner(asset_owner)
            .mint(mint)
            .authority(authority)
            .collection(collection)
            .minter_config(minter_config_pda)
            .project_config(project_config_pda);

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
