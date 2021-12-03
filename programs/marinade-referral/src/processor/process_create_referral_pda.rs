use anchor_lang::{prelude::*, solana_program::clock};
use marinade_finance::Fee;

use crate::{associated_token, constant::*, error::*, instructions::*};
use spl_associated_token_account::get_associated_token_address;

pub fn process_create_referral_pda(
    ctx: Context<CreateReferralPda>,
    _bump: u8,
    partner_name: [u8; 10],
) -> ProgramResult {
    // get beneficiary ATA
    let beneficiary_ata = get_associated_token_address(
        ctx.accounts.partner_account.key,
        &ctx.accounts.msol_mint.key(),
    );

    // check if beneficiary account address matches to partner_address and msol_mint
    if *ctx.accounts.beneficiary_account.key != beneficiary_ata {
        return Err(ReferralError::InvalidBeneficiaryAccount.into());
    }

    // create mSOL ATA for partner if it doesn't have yet
    if **ctx.accounts.beneficiary_account.lamports.borrow() == 0_u64 {
        associated_token::create(ctx.accounts.into_create_associated_token_account_ctx())?;
    }

    ctx.accounts.referral_state.partner_name = partner_name.clone();

    ctx.accounts.referral_state.beneficiary_account = *ctx.accounts.beneficiary_account.key;

    ctx.accounts.referral_state.transfer_duration = DEFAULT_TRANSFER_DURATION;
    ctx.accounts.referral_state.last_transfer_time = clock::Clock::get().unwrap().unix_timestamp;

    ctx.accounts.referral_state.deposit_sol_amount = 0;
    ctx.accounts.referral_state.deposit_sol_operations = 0;

    ctx.accounts.referral_state.deposit_stake_account_amount = 0;
    ctx.accounts.referral_state.deposit_stake_account_operations = 0;

    ctx.accounts.referral_state.liq_unstake_amount = 0;
    ctx.accounts.referral_state.liq_unstake_operations = 0;

    ctx.accounts.referral_state.delayed_unstake_amount = 0;
    ctx.accounts.referral_state.del_unstake_operations = 0;

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
