use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use marinade_finance::{
    instruction::LiquidUnstake as MarinadeLiquidUnstake, State as MarinadeSate,
};

use crate::account_structs::*;

pub fn process_liquid_unstake(ctx: Context<LiquidUnstake>, msol_amount: u64) -> ProgramResult {
    // accumulate treasury fees for the liquid-unstake
    // We are parsing manually to not make the IDL more complex with MarinadeSate
    let marinade_state: ProgramAccount<MarinadeSate> = ProgramAccount::try_from(
        &ctx.accounts.marinade_finance_program.key(),
        &ctx.accounts.state,
    )?;

    let max_lamports = ctx
        .accounts
        .liq_pool_sol_leg_pda
        .lamports()
        .saturating_sub(marinade_state.rent_exempt_for_token_acc);

    // fee is computed based on the liquidity *after* the user takes the sol
    let user_remove_lamports = marinade_state.calc_lamports_from_msol_amount(msol_amount)?;
    let liquid_unstake_fee = if user_remove_lamports >= max_lamports {
        // user is removing all liquidity
        marinade_state.liq_pool.lp_max_fee
    } else {
        let after_lamports = max_lamports - user_remove_lamports; //how much will be left?
        marinade_state.liq_pool.linear_fee(after_lamports)
    };

    // compute fee in msol
    let msol_fee = liquid_unstake_fee.apply(msol_amount);
    msg!("msol_fee {}", msol_fee);

    let is_treasury_msol_ready_for_transfer =
        marinade_state.check_treasury_msol_account(&ctx.accounts.treasury_msol_account)?;
    // cut 25% from the fee for the treasury
    let treasury_msol_cut = if is_treasury_msol_ready_for_transfer {
        marinade_state.liq_pool.treasury_cut.apply(msol_fee)
    } else {
        0
    };
    msg!("treasury_msol_cut {}", treasury_msol_cut);

    // liquid-unstake cpi
    let cpi_ctx = ctx.accounts.into_liquid_unstake_cpi_ctx();
    let cpi_accounts = cpi_ctx.to_account_metas(None);
    let data = MarinadeLiquidUnstake { msol_amount };
    let ix = Instruction {
        program_id: *cpi_ctx.program.key,
        accounts: cpi_accounts,
        data: data.data(),
    };
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            cpi_ctx.accounts.state,
            cpi_ctx.accounts.msol_mint,
            cpi_ctx.accounts.liq_pool_sol_leg_pda,
            cpi_ctx.accounts.liq_pool_msol_leg,
            cpi_ctx.accounts.treasury_msol_account,
            cpi_ctx.accounts.get_msol_from,
            cpi_ctx.accounts.get_msol_from_authority,
            cpi_ctx.accounts.transfer_sol_to,
            cpi_ctx.accounts.system_program,
            cpi_ctx.accounts.token_program,
            //
            ctx.accounts.marinade_finance_program.clone(),
        ],
        cpi_ctx.signer_seeds,
    )?;

    // update accumulators
    ctx.accounts.referral_state.liq_unstake_msol_fees = ctx
        .accounts
        .referral_state
        .liq_unstake_msol_fees
        .wrapping_add(treasury_msol_cut);
    ctx.accounts.referral_state.liq_unstake_amount = ctx
        .accounts
        .referral_state
        .liq_unstake_amount
        .wrapping_add(msol_amount);
    ctx.accounts.referral_state.liq_unstake_operations = ctx
        .accounts
        .referral_state
        .liq_unstake_operations
        .wrapping_add(1);

    Ok(())
}
