#![allow(unused_imports)]

use anchor_lang::prelude::*;

use crate::{constant::*, error::*, fees::Fee, instructions::*, states::*};

pub fn process_deposit(ctx: Context<Deposit>, lamports: u64) -> ProgramResult {
    ctx.accounts.state.deposit_sol_amount =
        ctx.accounts.state.deposit_sol_amount.wrapping_add(lamports);

    // TODO: cpi to marinade main program

    Ok(())
}
