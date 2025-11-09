use anchor_lang::prelude::*;

use mpl_core::{
    instructions::CreateCollectionV1CpiBuilder, 
    types::{Creator, PluginAuthorityPair, Royalties}
};

use crate::state::{CollectionConfig};

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: Collection account is validated by mpl_core program
    #[account(mut)]
    pub collection: Signer<'info>,
    #[account(
        init,
        space = 8 + CollectionConfig::INIT_SPACE,
        payer = owner,
        seeds = [b"collection_config", collection.key().as_ref()],
        bump,
    )]
    pub collection_config: Account<'info, CollectionConfig>,
    pub treasury: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
    /// CHECK: Verified by address constraint to mpl_core::ID
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> CreateCollection<'info>  {
    pub fn create_collection(
        &mut self,
        name: String,
        uri: String,
        mint_price: u64,
        royalties: u16,
        max_supply: u64,
        //assets_config: Option<AssetsConfig>,
        bump: &CreateCollectionBumps
    ) -> Result<()>{

        self.collection_config.set_inner(CollectionConfig {
            owner: self.owner.key(),
            collection: self.collection.key(),
            treasury: self.treasury.key(),
            mint_price,
            max_supply,
            //assets_config,
            bump: bump.collection_config
        });

        let seeds = &[
            b"collection_config",
            self.collection.to_account_info().key.as_ref(),
            &[self.collection_config.bump],
        ];

        let signer_seeds: &[&[&[u8]]] = &[&seeds[..]];

        let creator = Creator {
            address: self.treasury.key(),
            percentage: 100,
        };
        
        let royalties_plugin = Royalties {
            basis_points: royalties,
            creators: vec![creator],
            rule_set: mpl_core::types::RuleSet::None,
        };

        let plugin_pair = PluginAuthorityPair {
            plugin: mpl_core::types::Plugin::Royalties(royalties_plugin),
            authority: None,
        };

        CreateCollectionV1CpiBuilder::new(
            &self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .update_authority(Some(&self.collection_config.to_account_info()))
            .payer(&self.owner.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .plugins(vec![plugin_pair])
            .invoke_signed(signer_seeds)?;
        
        Ok(())
    }
}
