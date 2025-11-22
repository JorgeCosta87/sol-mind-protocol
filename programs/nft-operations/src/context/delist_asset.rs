use anchor_lang::prelude::*;
use mpl_core::{
    instructions::{RemovePluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{FreezeDelegate, Plugin, PluginType},
};

use crate::state::TradeHub;
use crate::Listing;

#[derive(Accounts)]
pub struct DelistAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    /// CHECK: Asset account used int trade hub PDA
    pub asset: UncheckedAccount<'info>,
    /// CHECK: Collection account will be verified by mpl core
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,
    #[account(
        mut,
        has_one = owner,
        close = owner,
        seeds = [
            b"listing",
            asset.key().as_ref(),
            trade_hub.key().as_ref(),
        ],
        bump,
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        seeds = [
            b"trade_hub",
            trade_hub.name.as_bytes(),
            trade_hub.project.key().as_ref(),
        ],
        bump,
    )]
    pub trade_hub: Account<'info, TradeHub>,

    pub system_program: Program<'info, System>,
    /// CHECK: Verified by address constraint to mpl_core::ID
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> DelistAsset<'info> {
    pub fn delist_asset(&mut self) -> Result<()> {
        let project_key = self.trade_hub.project.key();
        let seeds = &[
            b"trade_hub",
            self.trade_hub.name.as_bytes(),
            project_key.as_ref(),
            &[self.trade_hub.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program)
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|c| c.as_ref()))
            .payer(&self.payer.to_account_info())
            .authority(Some(&self.trade_hub.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(signer_seeds)?;

        RemovePluginV1CpiBuilder::new(&self.mpl_core_program)
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|c| c.as_ref()))
            .payer(&self.payer.to_account_info())
            .authority(Some(&self.owner.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin_type(PluginType::FreezeDelegate)
            .invoke()?;

        RemovePluginV1CpiBuilder::new(&self.mpl_core_program)
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|c| c.as_ref()))
            .payer(&self.payer.to_account_info())
            .authority(Some(&self.owner.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin_type(PluginType::TransferDelegate)
            .invoke()?;

        Ok(())
    }
}
