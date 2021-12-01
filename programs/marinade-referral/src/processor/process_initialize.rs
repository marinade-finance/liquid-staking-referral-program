use anchor_lang::prelude::*;

use crate::instructions::*;

pub fn process_initialize(ctx: Context<Initialize>, _bump: u8) -> ProgramResult {
    ctx.accounts.global_state.admin_account = *ctx.accounts.admin_account.key;

    Ok(())
}
