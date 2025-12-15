use anchor_lang::prelude::*;

use crate::{ComputeNodeInfo, ComputeNodeStatus};

#[derive(Accounts)]
#[instruction(node_pubkey: Pubkey)]
pub struct RegisterComputeNode<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + ComputeNodeInfo::INIT_SPACE,
        seeds = [b"compute_node", node_pubkey.as_ref()],
        bump,
    )]
    pub compute_node_info: Account<'info, ComputeNodeInfo>,

    pub system_program: Program<'info, System>,
}

impl<'info> RegisterComputeNode<'info> {
    pub fn register_compute_node(
        &mut self,
        node_pubkey: Pubkey,
        bumps: &RegisterComputeNodeBumps,
    ) -> Result<()> {
        self.compute_node_info.set_inner(ComputeNodeInfo {
            owner: self.owner.key(),
            node_pubkey: node_pubkey,
            node_info_cid: None,
            status: ComputeNodeStatus::Pending,
            bump: bumps.compute_node_info,
        });

        Ok(())
    }
}
