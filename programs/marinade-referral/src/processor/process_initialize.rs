#![allow(unused_imports)]

use anchor_lang::{prelude::*, solana_program::clock};

use crate::{constant::*, error::*, fees::Fee, instructions::*, states::*};

pub fn process_initialize(
    ctx: Context<Initialize>,
    _ref_code: String,
    _referral_bump: u8,
    _beneficiary_bump: u8,
) -> ProgramResult {
    ctx.accounts.state.partner_account = *ctx.accounts.partner_account.key;
    ctx.accounts.state.beneficiary_account = ctx.accounts.beneficiary.key();

    ctx.accounts.state.transfer_duration = DEFAULT_TRANSFER_DURATION;
    ctx.accounts.state.last_transfer_time = clock::Clock::get().unwrap().unix_timestamp;

    ctx.accounts.state.deposit_sol_amount = 0;
    ctx.accounts.state.depsoit_stake_account_amount = 0;

    ctx.accounts.state.liq_unstake_amount = 0;
    ctx.accounts.state.liq_unstake_operations = 0;

    ctx.accounts.state.del_unstake_amount = 0;
    ctx.accounts.state.del_unstake_operations = 0;

    ctx.accounts.state.base_fee = Fee {
        basis_points: DEFAULT_BASE_FEE_POINTS,
    };
    ctx.accounts.state.max_fee = Fee {
        basis_points: DEFAULT_MAX_FEE_POINTS,
    };
    ctx.accounts.state.max_net_stake_amount = DEFAULT_MAX_NET_STAKE_AMOUNT;

    ctx.accounts.state.pause = false;

    Ok(())
}
