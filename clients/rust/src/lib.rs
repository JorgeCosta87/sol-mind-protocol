#![allow(unused_imports)]

pub(crate) mod generated;

pub use generated::sol_mind_protocol::*;

pub mod token_manager {
    pub use super::generated::token_manager::*;
}

pub use generated::sol_mind_protocol::programs::SOL_MIND_PROTOCOL_ID;
pub use generated::token_manager::programs::TOKEN_MANAGER_ID;
