use std::str::FromStr;

use anchor_lang::{prelude::*, solana_program::clock};

use crate::{constant::*, error::*, instructions::*};

pub fn process_claim_transfer(ctx: Context<ClaimTransfer>) -> ProgramResult {
    // check emergency pause
    if ctx.accounts.referral_state.pause {
        return Err(ReferralError::Paused.into());
    }

    // verify msol_mint_authority
    if ctx.accounts.msol_mint.mint_authority.unwrap()
        != Pubkey::from_str(MSOL_MINT_AUTHORITY_ADDRESS).unwrap()
    {
        return Err(ReferralError::AccessDenied.into());
    }

    let current_time = clock::Clock::get().unwrap().unix_timestamp;
    let elapsed_time = current_time.wrapping_sub(ctx.accounts.referral_state.last_transfer_time);

    if elapsed_time as u32 > ctx.accounts.referral_state.transfer_duration {
        // TODO: transfer shared mSOL to partner

        // sets “Last transfer to partner timestamp“
        ctx.accounts.referral_state.last_transfer_time = current_time;

        // clears all accumulators
        ctx.accounts.referral_state.reset_liq_unstake_accumulators();
    } else {
        return Err(ReferralError::ClaimNotAvailable.into());
    }

    Ok(())
}
