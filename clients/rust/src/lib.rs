#![allow(unused_imports)]

pub(crate) mod generated;

pub use generated::sol_mind_protocol::*;

pub mod nft_operations {
    pub use super::generated::nft_operations::*;
}

pub mod dac_manager {
    pub use super::generated::dac_manager::*;
}

pub use generated::dac_manager::programs::DAC_MANAGER_ID;
pub use generated::nft_operations::programs::NFT_OPERATIONS_ID;
pub use generated::sol_mind_protocol::programs::SOL_MIND_PROTOCOL_ID;
