use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseAssetV1, fetch_plugin, instructions::{AddPluginV1CpiBuilder, ApprovePluginAuthorityV1CpiBuilder}, types::{FreezeDelegate, Plugin, PluginAuthority, PluginType, TransferDelegate}
};

use crate::{state::TradeHub};
use crate::{errors::ErrorCode, Listing};

#[derive(Accounts)]
pub struct ListAsset<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub owner: Signer<'info>,
    #[account(mut)]
    /// CHECK: Asset account manual verified
    pub asset: UncheckedAccount<'info>,
    /// CHECK: Collection account manual verified
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
        let transfer_delegate_plugint = fetch_plugin::<BaseAssetV1, TransferDelegate>(
            &self.asset.to_account_info(),
            PluginType::TransferDelegate
        );

        match transfer_delegate_plugint {
            Ok((authority, _, _)) => {
                if authority != PluginAuthority::Owner {
                    return err!(ErrorCode::AssetInvalidTransferAuthority)
                }

                ApprovePluginAuthorityV1CpiBuilder::new(&self.mpl_core_program)
                    .asset(&self.asset.to_account_info())
                    .collection(self.collection.as_ref().map(|c| c.as_ref()))
                    .payer(&self.payer.to_account_info())
                    .authority(Some(&self.owner.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin_type(PluginType::TransferDelegate)
                    .new_authority(
                        mpl_core::types::PluginAuthority::Address {
                            address: self.trade_hub.key()  
                        })
                    .invoke()?;
            },
            Err(_) => {
                AddPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
                    .asset(&self.asset.to_account_info())
                    .collection(self.collection.as_ref().map(|c| c.as_ref()))
                    .payer(&self.payer.to_account_info())
                    .authority(Some(&self.owner.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::TransferDelegate(TransferDelegate {}))
                    .init_authority(
                        mpl_core::types::PluginAuthority::Address {
                            address: self.trade_hub.key()  
                        })
                    .invoke()?;
            }
        }

        let freeze_delegate_result = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
            &self.asset.to_account_info(),
            PluginType::FreezeDelegate
        );
        
        match freeze_delegate_result {
            Ok(_) => return err!(ErrorCode::AssetAlreadyFrozen),
            Err(_) => {
                AddPluginV1CpiBuilder::new(&self.mpl_core_program)
                    .asset(&self.asset)
                    .collection(self.collection.as_ref().map(|c| c.as_ref()))
                    .payer(&self.payer)
                    .authority(Some(&self.owner))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
                    .init_authority(mpl_core::types::PluginAuthority::Address {
                        address: crate::ID,
                    })
                    .invoke()?;
            }
        }

        self.listing.set_inner(Listing {
            owner: self.owner.key(),
            asset: self.asset.key(),
            price,
            created_at: Clock::get()?.unix_timestamp,
            bump,
        });

        Ok(())
    }
}
