use anchor_lang::prelude::*;

use mpl_core::types::PluginAuthorityPair;
use mpl_core::{instructions::CreateV1CpiBuilder, types::DataState};
use sol_mind_protocol::helpers::pay_protocol_fee;

use crate::errors::ErrorCode;
use crate::state::MinterConfig;
use sol_mind_protocol::ProjectConfig;

#[derive(Accounts)]
pub struct MintAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        constraint = project_config.check_authorities(authority.key) @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub mint: Signer<'info>,
    /// CHECK: Collection account validated by mpl_core program
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,
    #[account(
        mut,
        seeds = [
            b"minter_config",
            project_config.key().as_ref(),
            minter_config.name.as_bytes(),
        ],
        bump = minter_config.bump,
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

impl<'info> MintAsset<'info> {
    pub fn mint_asset(
        &mut self,
        name: Option<String>,
        uri: Option<String>,
        plugins: Option<Vec<PluginAuthorityPair>>,
    ) -> Result<()> {
        if self.minter_config.max_supply > 0 {
            require!(
                self.minter_config.mints_counter < self.minter_config.max_supply,
                ErrorCode::MaxSupplyReached
            );
        }
        pay_protocol_fee(
            &self.payer,
            &self.protocol_config,
            &self.system_program,
            sol_mind_protocol::Operation::MintAsset,
            None,
        )?;

        let (asset_name, asset_uri) = match &self.minter_config.assets_config {
            Some(asset_config) => {
                let asset_name = format!(
                    "{} #{}",
                    asset_config.asset_name_prefix, self.minter_config.mints_counter
                );
                let asset_uri = format!(
                    "{}/{}/{}",
                    asset_config.asset_uri_prefix,
                    asset_config.asset_name_prefix,
                    self.minter_config.mints_counter
                );
                (asset_name, asset_uri)
            }
            None => {
                let asset_name = name.ok_or(error!(ErrorCode::RequireNameAnddUri))?;
                let asset_uri = uri.ok_or(error!(ErrorCode::RequireNameAnddUri))?;
                (asset_name, asset_uri)
            }
        };

        let payer_info = self.payer.to_account_info();
        let owner = self.owner.to_account_info();
        let mpl_core_program_info = self.mpl_core_program.to_account_info();
        let mint_info = self.mint.to_account_info();
        let minter_config_info = self.minter_config.to_account_info();
        let system_program_info = self.system_program.to_account_info();

        let mut builder = CreateV1CpiBuilder::new(&mpl_core_program_info);
        builder
            .asset(&mint_info)
            .payer(&payer_info)
            .owner(Some(&owner))
            .system_program(&system_program_info)
            .data_state(DataState::AccountState)
            .name(asset_name)
            .uri(asset_uri);

        if let Some(collection) = &self.collection {
            require!(
                collection.key() == self.minter_config.collection.unwrap(),
                ErrorCode::CollectionMismatch
            );
            builder
                .authority(Some(&minter_config_info))
                .collection(Some(collection.as_ref()));
        } else {
            builder.authority(None).update_authority(None);
        }

        if let Some(plugins) = plugins {
            builder.plugins(plugins);
        }

        let project_config_key = self.project_config.key();
        let name_bytes = self.minter_config.name.as_bytes();
        let seeds = &[
            b"minter_config",
            project_config_key.as_ref(),
            name_bytes,
            &[self.minter_config.bump],
        ];

        let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

        builder.invoke_signed(signer_seeds)?;

        self.minter_config.mints_counter = self
            .minter_config
            .mints_counter
            .checked_add(1)
            .expect("Mints counter overflowed");

        Ok(())
    }
}
