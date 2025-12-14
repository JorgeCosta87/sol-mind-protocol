use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::{ComputeNodeInfo, ComputeNodeStatus};

#[derive(Accounts)]
pub struct ClaimComputeNode<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub compute_node: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"compute_node",
            compute_node_info.node_pubkey.as_ref(),
        ],
        bump = compute_node_info.bump,
    )]
    pub compute_node_info: Account<'info, ComputeNodeInfo>,

    pub system_program: Program<'info, System>,
}

impl<'info> ClaimComputeNode<'info> {
    pub fn claim_compute_node(&mut self, node_info_cid: String) -> Result<()> {
        require!(
            self.compute_node.key() == self.compute_node_info.node_pubkey,
            ErrorCode::Unauthorized
        );
        require!(
            self.compute_node_info.status == ComputeNodeStatus::Pending,
            ErrorCode::InvalidNodeStatus
        );

        self.compute_node_info.node_info_cid = Some(node_info_cid);
        self.compute_node_info.status = ComputeNodeStatus::Approved;

        Ok(())
    }
}