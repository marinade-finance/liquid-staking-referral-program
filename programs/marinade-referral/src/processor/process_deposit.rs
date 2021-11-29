use anchor_lang::prelude::*;

use crate::instructions::*;

pub fn process_deposit(ctx: Context<Deposit>, lamports: u64) -> ProgramResult {
    ctx.accounts.state.deposit_sol_amount =
        ctx.accounts.state.deposit_sol_amount.wrapping_add(lamports);
    ctx.accounts.state.deposit_sol_operations =
        ctx.accounts.state.deposit_sol_operations.wrapping_add(1);

    // TODO: cpi to Marinade main program

    Ok(())
}
