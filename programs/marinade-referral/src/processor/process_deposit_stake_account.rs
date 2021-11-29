use anchor_lang::prelude::*;

use crate::instructions::*;

pub fn process_deposit_stake_account(
    ctx: Context<DepositStakeAccount>,
    validator_index: u32,
) -> ProgramResult {
    // TODO: confirm workflow
    ctx.accounts.state.depsoit_stake_account_amount = ctx
        .accounts
        .state
        .depsoit_stake_account_amount
        .wrapping_add(**ctx.accounts.stake_account.lamports.borrow());

    // TODO: cpi to Marinade main program

    Ok(())
}
