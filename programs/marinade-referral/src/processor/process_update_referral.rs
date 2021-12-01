use anchor_lang::prelude::*;

use crate::{error::*, instructions::*};

pub fn process_update_referral(
    ctx: Context<UpdateReferral>,
    transfer_duration: u32,
    pause: bool,
) -> ProgramResult {
    // check authority
    if ctx
        .accounts
        .global_state
        .admin_account
        .ne(ctx.accounts.admin_account.key)
    {
        return Err(ReferralError::AccessDenied.into());
    }

    ctx.accounts.referral_state.transfer_duration = transfer_duration;
    ctx.accounts.referral_state.pause = pause;

    Ok(())
}
