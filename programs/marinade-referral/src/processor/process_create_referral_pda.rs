use anchor_lang::{prelude::*, solana_program::clock};
use marinade_finance::Fee;

use crate::{associated_token, constant::*, instructions::*};

pub fn process_create_referral_pda(
    ctx: Context<CreateReferralPda>,
    _bump: u8,
    partner_name: [u8; 10],
) -> ProgramResult {
    // create associated token account for partner
    // TODO: validate beneficiary ATA in a better way
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

    ctx.accounts.referral_state.del_unstake_amount = 0;
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
