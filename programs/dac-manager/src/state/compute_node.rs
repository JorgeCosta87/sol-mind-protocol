use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum ComputeNodeStatus {
    Pending,
    Approved,
    Rejected,
    Disabled,
}

#[account]
#[derive(InitSpace)]
pub struct ComputeNodeInfo {
    pub owner: Pubkey,
    pub node_pubkey: Pubkey,
    #[max_len(32)]
    pub node_info_cid: Option<String>,
    pub status: ComputeNodeStatus,
    pub bump: u8,
}
