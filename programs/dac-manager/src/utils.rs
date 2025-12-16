use anchor_lang::prelude::*;
use anchor_lang::system_program;


use crate::errors::ErrorCode;

pub fn init_dynamic_pda<'info>(
    payer: &Signer<'info>,
    target_account: &AccountInfo<'info>,
    seeds: &[&[u8]],
    space: usize,
    owner: &Pubkey,
    system_program: &Program<'info, System>,
) -> Result<(u8)> {
    let (pda, bump) = Pubkey::find_program_address(seeds, &crate::ID);
    require_keys_eq!(target_account.key(), pda, ErrorCode::InvalidGoalPDA);

    if target_account.lamports() > 0 && !target_account.data_is_empty() {
         return Err(ErrorCode::AccountAlreadyInitialized.into());
    }

    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(space);

    let bump_seed = &[bump];
    let mut signer_seeds = seeds.to_vec();
    signer_seeds.push(bump_seed);
    let signer_seeds = &[&signer_seeds[..]];

    let cpi_accounts = system_program::CreateAccount {
        from: payer.to_account_info(),
        to: target_account.clone(),
    };
    let cpi_context = CpiContext::new_with_signer(
        system_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    system_program::create_account(
        cpi_context,
        required_lamports,
        space as u64,
        owner,
    )?;

    Ok((bump))
}
