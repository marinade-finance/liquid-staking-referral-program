use anchor_lang::prelude::*;

use crate::account_structs::*;

pub fn process_initialize(ctx: Context<Initialize>) -> ProgramResult {
    ctx.accounts.global_state.admin_account = *ctx.accounts.admin_account.key;
    ctx.accounts.global_state.payment_mint = *ctx.accounts.payment_mint.key;
    Ok(())
}
