use anchor_lang::prelude::*;

use mpl_core::{instructions::CreateCollectionV1CpiBuilder, types::PluginAuthorityPair};
use sol_mind_protocol::helpers::pay_protocol_fee;

use crate::errors::ErrorCode;
use crate::state::{AssetsConfig, MinterConfig};
use sol_mind_protocol::ProjectConfig;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateMinterConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        constraint = project_config.check_authorities(authority.key) @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,
    /// CHECK: Collection account is validated by mpl_core program (optional)
    #[account(mut)]
    pub collection: Option<Signer<'info>>,
    #[account(
        init,
        space = 8 + MinterConfig::INIT_SPACE,
        payer = payer,
        seeds = [
            b"minter_config",
            project_config.key().as_ref(),
            name.as_bytes(),
        ],
        bump,
    )]
    pub minter_config: Account<'info, MinterConfig>,
    #[account(
        seeds = [
            b"project",
            project_config.owner.as_ref(),
            project_config.project_id.to_le_bytes().as_ref(),
        ],
        bump = project_config.bump,
        seeds::program = sol_mind_protocol::ID,
    )]
    pub project_config: Account<'info, ProjectConfig>,
    #[account(
        mut,
        seeds = [b"sol-mind-protocol"],
        bump = protocol_config.bump,
        seeds::program = sol_mind_protocol::ID,
    )]
    pub protocol_config: Account<'info, sol_mind_protocol::ProtocolConfig>,

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
        pay_protocol_fee(
            &self.payer,
            &self.protocol_config,
            &self.system_program,
            sol_mind_protocol::Operation::CreateMinterConfig,
            None,
        )?;

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

            let project_config_key = self.project_config.key();
            let name_bytes = name.as_bytes();
            let seeds = &[
                b"minter_config",
                project_config_key.as_ref(),
                name_bytes,
                &[self.minter_config.bump],
            ];

            let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

            CreateCollectionV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .collection(&collection.to_account_info())
                .update_authority(Some(&self.minter_config.to_account_info())) // this should be none to be imutable forever?
                .payer(&self.payer.to_account_info())
                .system_program(&self.system_program.to_account_info())
                .name(name.clone())
                .uri(uri)
                .plugins(plugins)
                .invoke_signed(signer_seeds)?;
        }

        Ok(())
    }
}
