use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct AssetsConfig {
    #[max_len(64)]
    pub asset_name_prefix: String,
    #[max_len(200)]
    pub asset_uri_prefix: String,
}

#[account]
#[derive(InitSpace)]
pub struct MinterConfig {
    pub collection: Option<Pubkey>,
    pub mint_price: u64,
    pub max_supply: u64, // if 0 unlimited
    pub assets_config: Option<AssetsConfig>,
    pub bump: u8,
}

