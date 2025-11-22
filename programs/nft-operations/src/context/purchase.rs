use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use mpl_core::{
    instructions::{TransferV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{FreezeDelegate, Plugin},
};
use sol_mind_protocol::{helpers::pay_protocol_fee, Operation, ProjectConfig};

use crate::errors::ErrorCode;
use crate::state::{Listing, TradeHub};

#[derive(Accounts)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub owner: SystemAccount<'info>,
    #[account(mut)]
    /// CHECK: Asset account manual verified
    pub asset: UncheckedAccount<'info>,
    /// CHECK: Collection account validated by mpl_core program
    #[account(mut)]
    pub collection: Option<UncheckedAccount<'info>>,
    #[account(
        mut,
        has_one = owner @ ErrorCode::NotAssetOwner,
        close = owner,
        seeds = [
            b"listing",
            asset.key().as_ref(),
            trade_hub.key().as_ref(),
        ],
        bump = listing.bump,
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
    #[account(
        mut,
        seeds = [b"treasury", project_config.key().as_ref()],
        bump = project_config.treasury_bump,
        seeds::program = sol_mind_protocol::ID,
    )]
    pub treasury: SystemAccount<'info>,
    #[account(
        seeds = [
            b"project",
            project_config.owner.as_ref(),
            project_config.project_id.to_le_bytes().as_ref(),
            project_config.protocol_config.as_ref(),
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

impl<'info> Purchase<'info> {
    pub fn purchase_asset(&mut self) -> Result<()> {
        let asset_price = self.listing.price;

        pay_protocol_fee(
            &self.buyer,
            &self.protocol_config,
            &self.system_program,
            Operation::TradeNFT,
            Some(asset_price),
        )?;

        let protocol_fee = self
            .protocol_config
            .calculate_fee_amount(Operation::TradeNFT, Some(asset_price))?;
        let trade_hub_fee = self.trade_hub.calculate_fee_amount(self.listing.price)?;

        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.treasury.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, trade_hub_fee)?;

        let seller_amount = asset_price
            .checked_sub(protocol_fee)
            .unwrap()
            .checked_sub(trade_hub_fee)
            .unwrap();

        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.owner.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, seller_amount)?;

        let project_key = self.trade_hub.project.key();
        let seeds = &[
            b"trade_hub",
            self.trade_hub.name.as_bytes(),
            project_key.as_ref(),
            &[self.trade_hub.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program)
            .asset(&self.asset)
            .collection(self.collection.as_ref().map(|c| c.as_ref()))
            .payer(&self.buyer.to_account_info())
            .authority(Some(&self.trade_hub.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(signer_seeds)?;

        TransferV1CpiBuilder::new(&self.mpl_core_program)
            .asset(&self.asset)
            .collection(self.collection.as_ref().map(|c| c.as_ref()))
            .payer(&self.buyer.to_account_info())
            .authority(Some(&self.trade_hub.to_account_info()))
            .new_owner(&self.buyer.to_account_info())
            .invoke_signed(signer_seeds)?;
        Ok(())
    }
}
