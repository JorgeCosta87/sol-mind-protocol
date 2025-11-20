#[path = "sol-mind-protocol/mod.rs"]
pub(crate) mod sol_mind_protocol;

#[path = "token-manager/mod.rs"]
pub(crate) mod token_manager;

pub mod types {
    pub use super::sol_mind_protocol::types::*;
    pub use super::token_manager::types::AssetsConfig;
}
