use anchor_lang::error_code;

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized: Only the owner can perform this action")]
    Unauthorized,
    #[msg("Task is not ready to be submitted")]
    InvalidTaskStatus,
    #[msg("Agent is not public")]
    AgentNotPublic,
    #[msg("Node is not pending")]
    InvalidNodeStatus,
    #[msg("Compute node is not approved")]
    ComputeNodeNotApproved,
    #[msg("Compute node does not match the provided public key")]
    ComputeNodeMismatch,
    #[msg("Agent is not public or the owner of the compute node")]
    AgentNotPublicOrComputeNodeOwner,
}
