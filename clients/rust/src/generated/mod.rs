#[path = "sol-mind-protocol/mod.rs"]
pub(crate) mod sol_mind_protocol;

#[path = "nft-operations/mod.rs"]
pub(crate) mod nft_operations;

pub mod types {
    pub use super::nft_operations::types::AssetsConfig;
    pub use super::sol_mind_protocol::types::*;
}
