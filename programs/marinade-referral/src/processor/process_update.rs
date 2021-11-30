use anchor_lang::prelude::*;

use crate::{error::*, instructions::*};

pub fn process_update(ctx: Context<Update>, transfer_duration: u32, pause: bool) -> ProgramResult {
    // check authority
    if ctx
        .accounts
        .state
        .admin_account
        .ne(ctx.accounts.admin_account.key)
    {
        return Err(ReferralError::AccessDenied.into());
    }

    ctx.accounts.state.transfer_duration = transfer_duration;
    ctx.accounts.state.pause = pause;

    Ok(())
}
