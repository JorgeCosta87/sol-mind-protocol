use anchor_lang::prelude::*;
use mpl_core::{
    instructions::AddPluginV1CpiBuilder,
    types::{Plugin, TransferDelegate},
};

use crate::state::TradeHub;
use crate::Listing;

#[derive(Accounts)]
pub struct ListAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub owner: Signer<'info>,
    #[account(mut)]
    /// CHECK: Collection account validated by mpl_core program
    pub asset: UncheckedAccount<'info>,
    /// CHECK: Collection account validated by mpl_core program
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,
    #[account(
        init,
        payer = payer,
        space = 8 + Listing::INIT_SPACE,
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

impl<'info> ListAsset<'info> {
    pub fn create_listing(&mut self, price: u64, bump: u8) -> Result<()> {
        self.listing.set_inner(Listing {
            owner: self.owner.key(),
            asset: self.asset.key(),
            price,
            created_at: Clock::get()?.unix_timestamp,
            bump,
        });

        AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(self.collection.as_ref().map(|c| c.as_ref()))
            .payer(&self.payer.to_account_info())
            .authority(Some(&self.owner.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::TransferDelegate(TransferDelegate {}))
            .invoke()?;

        Ok(())
    }
}
