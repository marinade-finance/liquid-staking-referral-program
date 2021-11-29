use anchor_lang::prelude::*;

use crate::instructions::*;

pub fn process_liquid_unstake(ctx: Context<LiquidUnstake>, msol_amount: u64) -> ProgramResult {
    ctx.accounts.state.liq_unstake_amount = ctx
        .accounts
        .state
        .liq_unstake_amount
        .wrapping_add(msol_amount);
    ctx.accounts.state.liq_unstake_operations =
        ctx.accounts.state.liq_unstake_operations.wrapping_add(1);

    // TODO: cpi to Marinade main program

    Ok(())
}
