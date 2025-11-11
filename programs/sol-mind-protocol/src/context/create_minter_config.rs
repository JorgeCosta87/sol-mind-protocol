use anchor_lang::prelude::*;

use mpl_core::{
    instructions::CreateCollectionV1CpiBuilder, 
    types::{PluginAuthorityPair}
};

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
    /// CHECK: Collection account is validated by mpl_core program (optional)
    #[account(
        init,
        space = 8 + MinterConfig::INIT_SPACE,
        payer = payer,
        seeds = [
            b"minter_config",
            project_config.project_id.to_le_bytes().as_ref(),
            project_config.increament_minter_config_counter().to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub minter_config: Account<'info, MinterConfig>,
    #[account(mut)]
    pub project_config: Account<'info, ProjectConfig>,
    #[account(mut)]
    pub collection: Option<Signer<'info>>,
    
    pub system_program: Program<'info, System>,
    /// CHECK: Verified by address constraint to mpl_core::ID
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> CreateMinterConfig<'info> {
    pub fn create_minter_config(
        &mut self,
        name: String,
        uri: String,
        mint_price: u64,
        max_supply: u64,
        plugins: Option<Vec<PluginAuthorityPair>>,
        assets_config: Option<AssetsConfig>,
        bump: &CreateMinterConfigBumps,
    ) -> Result<()> {
        self.minter_config.set_inner(MinterConfig {
            collection: self.collection.as_ref().map(|c| c.key()),
            mint_price,
            max_supply,
            assets_config,
            bump: bump.minter_config,
        });


        if let Some(collection) = &self.collection {
            let project_id_bytes = self.project_config.project_id.to_le_bytes();
            let counter_bytes = self.project_config.minter_config_counter.to_le_bytes();
            let seeds = &[
                b"minter_config",
                project_id_bytes.as_ref(),
                counter_bytes.as_ref(),
                &[self.minter_config.bump],
            ];

            let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

        CreateCollectionV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                .collection(&collection.to_account_info())
                .update_authority(Some(&self.minter_config.to_account_info()))
                .payer(&self.authority.to_account_info())
                .system_program(&self.system_program.to_account_info())
                .name(name)
                .uri(uri)
                .plugins(plugins.unwrap())
                .invoke_signed(signer_seeds)?;
        }
        
        Ok(())
    }
}
