use anchor_lang::{prelude::*, solana_program::clock};

use crate::{associated_token, constant::*, fees::Fee, instructions::*};

pub fn process_initialize(ctx: Context<Initialize>, partner_name: [u8; 10]) -> ProgramResult {
    ctx.accounts.state.partner_name = partner_name.clone();

    ctx.accounts.state.partner_account = *ctx.accounts.partner_account.key;
    ctx.accounts.state.beneficiary_account = *ctx.accounts.beneficiary_account.key;

    ctx.accounts.state.transfer_duration = DEFAULT_TRANSFER_DURATION;
    ctx.accounts.state.last_transfer_time = clock::Clock::get().unwrap().unix_timestamp;

    ctx.accounts.state.deposit_sol_amount = 0;
    ctx.accounts.state.deposit_sol_operations = 0;

    ctx.accounts.state.deposit_stake_account_amount = 0;
    ctx.accounts.state.deposit_stake_account_operations = 0;

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
    ctx.accounts.state.max_net_stake = DEFAULT_MAX_NET_STAKE;

    ctx.accounts.state.pause = false;

    // create associated token account for partner
    if **ctx.accounts.beneficiary_account.lamports.borrow() == 0_u64 {
        associated_token::create(ctx.accounts.into_create_associated_token_account_ctx())?;
    }

    Ok(())
}
