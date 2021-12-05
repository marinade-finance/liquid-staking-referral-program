use anchor_lang::{prelude::*, solana_program::clock};
use marinade_finance::Fee;

use crate::error::*;
use crate::{account_structs::*, associated_token, constant::*};
use spl_associated_token_account::get_associated_token_address;

pub fn process_init_referral_account(
    ctx: Context<InitReferralAccount>,
    partner_name: String,
) -> ProgramResult {
    msg!("process_init_referral_account");
    // // get beneficiary ATA
    // let beneficiary_ata = get_associated_token_address(
    //     ctx.accounts.partner_account.key,
    //     &ctx.accounts.payment_mint.key(),
    // );

    // check if beneficiary account address matches to partner_address and msol_mint
    if ctx.accounts.token_partner_account.owner != *ctx.accounts.partner_account.key {
        return Err(ReferralError::InvalidBeneficiaryAccountOwner.into());
    }
    if ctx.accounts.token_partner_account.mint != ctx.accounts.payment_mint.key() {
        return Err(ReferralError::InvalidBeneficiaryAccountMint.into());
    }

    // // create mSOL ATA for partner if it is not created yet
    // if **ctx.accounts.token_partner_account.lamports.borrow() == 0_u64 {
    //     associated_token::create(ctx.accounts.into_create_associated_token_account_ctx())?;
    // }

    ctx.accounts.referral_state.partner_name = partner_name.clone();

    ctx.accounts.referral_state.partner_account = ctx.accounts.partner_account.key();
    ctx.accounts.referral_state.token_partner_account = ctx.accounts.token_partner_account.key();

    ctx.accounts.referral_state.transfer_duration = DEFAULT_TRANSFER_DURATION;
    ctx.accounts.referral_state.last_transfer_time = clock::Clock::get().unwrap().unix_timestamp;

    ctx.accounts.referral_state.deposit_sol_amount = 0;
    ctx.accounts.referral_state.deposit_sol_operations = 0;

    ctx.accounts.referral_state.deposit_stake_account_amount = 0;
    ctx.accounts.referral_state.deposit_stake_account_operations = 0;

    ctx.accounts.referral_state.liq_unstake_amount = 0;
    ctx.accounts.referral_state.liq_unstake_operations = 0;

    ctx.accounts.referral_state.delayed_unstake_amount = 0;
    ctx.accounts.referral_state.delayed_unstake_operations = 0;

    ctx.accounts.referral_state.base_fee = Fee {
        basis_points: DEFAULT_BASE_FEE_POINTS,
    };
    ctx.accounts.referral_state.max_fee = Fee {
        basis_points: DEFAULT_MAX_FEE_POINTS,
    };
    ctx.accounts.referral_state.max_net_stake = DEFAULT_MAX_NET_STAKE;

    ctx.accounts.referral_state.pause = false;

    Ok(())
}
