use anchor_lang::prelude::*;

use mpl_core::{
    instructions::CreateV1CpiBuilder,
    types::DataState,
};

use crate::state::{ProjectConfig, MinterConfig};

#[derive(Accounts)]
pub struct MintAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        constraint = project_config.check_authorities(authority.key)
    
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub mint: Signer<'info>,
    #[account(
        seeds = [
            b"minter_config",
            project_config.project_id.to_le_bytes().as_ref(),
            project_config.increament_minter_config_counter().to_le_bytes().as_ref()
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
    )]
    pub project_config: Account<'info, ProjectConfig>,
    /// CHECK: Collection account validated by mpl_core program
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,

    pub system_program: Program<'info, System>,
    /// CHECK: Verified by address constraint to mpl_core::ID
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> MintAsset<'info>  {
    pub fn mint_asset(
        &mut self,
        name: String,
        uri: String,
    ) -> Result<()>{
        let project_id_bytes = self.project_config.project_id.to_le_bytes();
        let counter_bytes = self.project_config.minter_config_counter.to_le_bytes();
        let seeds = &[
            b"minter_config",
            project_id_bytes.as_ref(),
            counter_bytes.as_ref(),
            &[self.minter_config.bump],
        ];

        let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

        let mpl_core_program_info = self.mpl_core_program.to_account_info();
        let mint_info = self.mint.to_account_info();
        let authority_info = self.authority.to_account_info();
        let payer_info = self.payer.to_account_info();
        let system_program_info = self.system_program.to_account_info();

        let mut builder = CreateV1CpiBuilder::new(&mpl_core_program_info);
        builder
            .asset(&mint_info)
            .authority(Some(&authority_info))
            .payer(&payer_info)
            .owner(Some(&payer_info))
            .update_authority(None)
            .system_program(&system_program_info)
            .data_state(DataState::AccountState)
            .name(name)
            .uri(uri);

        if let Some(collection) = &self.collection {
            builder.collection(Some(collection.as_ref()));
        }

        builder.invoke_signed(signer_seeds)?;
        
        Ok(())
    }
}
