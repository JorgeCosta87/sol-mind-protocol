use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct AssetsConfig {
    #[max_len(32)]
    pub asset_name_prefix: String,
    #[max_len(200)]
    pub asset_uri_prefix: String,
}

#[account]
#[derive(InitSpace)]
pub struct MinterConfig {
    #[max_len(32)]
    pub name: String,
    pub mint_price: u64,
    pub mints_counter: u64,
    pub max_supply: u64, // if 0 unlimited
    pub assets_config: Option<AssetsConfig>,
    pub collection: Option<Pubkey>,
    pub bump: u8,
}

