use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use marinade_finance::instruction::LiquidUnstake as MarinadeLiquidUnstake;

use crate::instructions::*;

pub fn process_liquid_unstake(ctx: Context<LiquidUnstake>, msol_amount: u64) -> ProgramResult {
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
            cpi_ctx.accounts.get_msol_from,
            cpi_ctx.accounts.get_msol_from_authority,
            cpi_ctx.accounts.transfer_sol_to,
            cpi_ctx.accounts.treasury_msol_account,
            cpi_ctx.accounts.system_program,
            cpi_ctx.accounts.token_program,
        ],
        cpi_ctx.signer_seeds,
    )?;

    // update accumulators
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
