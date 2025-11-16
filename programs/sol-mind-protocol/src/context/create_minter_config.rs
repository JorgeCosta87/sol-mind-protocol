use anchor_lang::prelude::*;

use mpl_core::{instructions::CreateCollectionV1CpiBuilder, types::PluginAuthorityPair};

use crate::errors::ErrorCode;
use crate::state::{AssetsConfig, MinterConfig, ProjectConfig};

#[derive(Accounts)]
pub struct CreateMinterConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        constraint = project_config.check_authorities(authority.key)
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub collection: Option<Signer<'info>>,
    /// CHECK: Collection account is validated by mpl_core program (optional)
    #[account(
        init,
        space = 8 + MinterConfig::INIT_SPACE,
        payer = payer,
        seeds = [
            b"minter_config",
            project_config.project_id.to_le_bytes().as_ref(),
            project_config.minter_config_next().to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub minter_config: Account<'info, MinterConfig>,
    #[account(mut)]
    pub project_config: Account<'info, ProjectConfig>,

    pub system_program: Program<'info, System>,
    /// CHECK: Verified by address constraint to mpl_core::ID
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> CreateMinterConfig<'info> {
    pub fn create_minter_config(
        &mut self,
        name: String,
        mint_price: u64,
        max_supply: u64,
        assets_config: Option<AssetsConfig>,
        uri: Option<String>,
        plugins: Option<Vec<PluginAuthorityPair>>,
        bump: &CreateMinterConfigBumps,
    ) -> Result<()> {
        self.project_config.minter_config_counter = self.project_config.minter_config_next();

        self.minter_config.set_inner(MinterConfig {
            name: name.clone(),
            mint_price,
            mints_counter: 0,
            max_supply,
            assets_config,
            collection: self.collection.as_ref().map(|c| c.key()),
            bump: bump.minter_config,
        });

        if let Some(collection) = &self.collection {
            let uri = uri.ok_or(error!(ErrorCode::RequiredUri))?;
            let plugins = plugins.unwrap_or_default();

            let project_id_bytes = self.project_config.project_id.to_le_bytes();
            let minter_counter_bytes = self.project_config.minter_config_counter.to_le_bytes();
            let seeds = &[
                b"minter_config",
                project_id_bytes.as_ref(),
                minter_counter_bytes.as_ref(),
                &[self.minter_config.bump],
            ];

            let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

            CreateCollectionV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .collection(&collection.to_account_info())
                .update_authority(Some(&self.minter_config.to_account_info())) // this should be none to be imutable forever?
                .payer(&self.payer.to_account_info())
                .system_program(&self.system_program.to_account_info())
                .name(name)
                .uri(uri)
                .plugins(plugins)
                .invoke_signed(signer_seeds)?;
        }

        Ok(())
    }
}
