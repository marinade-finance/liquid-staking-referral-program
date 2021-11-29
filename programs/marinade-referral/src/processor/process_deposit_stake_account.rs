use anchor_lang::prelude::*;

use crate::instructions::*;

pub fn process_deposit_stake_account(
    ctx: Context<DepositStakeAccount>,
    validator_index: u32,
) -> ProgramResult {
    // TODO: confirm workflow
    ctx.accounts.state.deposit_stake_account_amount = ctx
        .accounts
        .state
        .deposit_stake_account_amount
        .wrapping_add(**ctx.accounts.stake_account.lamports.borrow());
    ctx.accounts.state.deposit_stake_account_operations = ctx
        .accounts
        .state
        .deposit_stake_account_operations
        .wrapping_add(1);

    // TODO: cpi to Marinade main program

    Ok(())
}
