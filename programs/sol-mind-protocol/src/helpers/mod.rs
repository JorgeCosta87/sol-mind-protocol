use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::{Operation, ProtocolConfig};

pub fn pay_protocol_fee<'info>(
    owner: &Signer<'info>,
    protocol_config: &Account<'info, ProtocolConfig>,
    operation: Operation,
    base_amount: Option<u64>,
    system_program: &Program<'info, System>,
) -> Result<()> {
    let fee_amount = protocol_config.calculate_fee_amount(operation, base_amount);

    if fee_amount > 0 {
        let cpi_program = system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: owner.to_account_info(),
            to: protocol_config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, fee_amount)?;
    }

    Ok(())
}
