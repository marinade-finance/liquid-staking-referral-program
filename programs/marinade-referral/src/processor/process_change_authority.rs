use anchor_lang::prelude::*;

use crate::{error::*, instructions::*};

///change admin
pub fn process_change_authority(ctx: Context<ChangeAuthority>) -> ProgramResult {
    // check authority
    if ctx
        .accounts
        .global_state
        .admin_account
        .ne(ctx.accounts.admin_account.key)
    {
        return Err(ReferralError::AccessDenied.into());
    }

    ctx.accounts.global_state.admin_account = *ctx.accounts.new_admin_account.key;

    Ok(())
}
