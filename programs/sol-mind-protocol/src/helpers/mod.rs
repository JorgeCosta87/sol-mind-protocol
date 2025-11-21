use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::{errors::ProtocolError, Operation, ProtocolConfig};

pub fn validate_transfer(account_info: &AccountInfo, amount: u64) -> Result<()> {
    let current_balance = account_info.lamports();
    let rent_exempt = Rent::get()?.minimum_balance(account_info.data_len());

    let remaining_balance = current_balance
        .checked_sub(amount)
        .ok_or(ProtocolError::InsufficientFunds)?;

    require!(
        remaining_balance >= rent_exempt,
        ProtocolError::MinimumBalanceRequired
    );

    Ok(())
}

pub fn cpi_transfer<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    amount: u64,
    system_program: &Program<'info, System>,
) -> Result<()> {
    let cpi_program = system_program.to_account_info();
    let cpi_accounts = Transfer { from: from, to: to };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_ctx, amount)?;

    Ok(())
}

pub fn pay_protocol_fee<'info>(
    fee_payer: &Signer<'info>,
    protocol_config: &Account<'info, ProtocolConfig>,
    system_program: &Program<'info, System>,
    operation: Operation,
    base_amount: Option<u64>,
) -> Result<()> {
    let fee_amount = protocol_config.calculate_fee_amount(operation, base_amount);

    if fee_amount > 0 {
        cpi_transfer(
            fee_payer.to_account_info(),
            protocol_config.to_account_info(),
            fee_amount,
            system_program,
        )?;
    }

    Ok(())
}
