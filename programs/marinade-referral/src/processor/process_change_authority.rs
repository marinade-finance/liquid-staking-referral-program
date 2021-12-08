use anchor_lang::prelude::*;

use crate::account_structs::*;

///change admin
pub fn process_change_authority(ctx: Context<ChangeAuthority>) -> ProgramResult {
    ctx.accounts.global_state.admin_account = *ctx.accounts.new_admin_account.key;

    Ok(())
}
