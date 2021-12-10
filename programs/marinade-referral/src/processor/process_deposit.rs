use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use marinade_finance::instruction::Deposit as MarinadeDeposit;

use crate::account_structs::*;

pub fn process_deposit(ctx: Context<Deposit>, lamports: u64) -> ProgramResult {
    msg!(
        "enter process_deposit {}",
        ctx.accounts.transfer_from.lamports()
    );
    let cpi_ctx = ctx.accounts.into_marinade_deposit_cpi_ctx();
    let cpi_accounts = cpi_ctx.to_account_metas(None);
    let data = MarinadeDeposit { lamports };
    let ix = Instruction {
        program_id: *ctx.accounts.marinade_finance_program.key,
        accounts: cpi_accounts,
        data: data.data(),
    };
    msg!("calling MarinadeDeposit");
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            cpi_ctx.accounts.state,
            cpi_ctx.accounts.msol_mint,
            cpi_ctx.accounts.liq_pool_sol_leg_pda,
            cpi_ctx.accounts.liq_pool_msol_leg,
            cpi_ctx.accounts.liq_pool_msol_leg_authority,
            cpi_ctx.accounts.reserve_pda,
            cpi_ctx.accounts.transfer_from,
            cpi_ctx.accounts.mint_to,
            cpi_ctx.accounts.msol_mint_authority,
            cpi_ctx.accounts.system_program,
            cpi_ctx.accounts.token_program,
            //
            ctx.accounts.marinade_finance_program.clone(),
        ],
        cpi_ctx.signer_seeds,
    )?;
    msg!(
        "after MarinadeDeposit {}",
        ctx.accounts.transfer_from.lamports()
    );
    msg!(
        "deposit-SOL: update accumulators, deposit-lamports {}",
        lamports
    );
    // update accumulators
    ctx.accounts.referral_state.deposit_sol_amount = ctx
        .accounts
        .referral_state
        .deposit_sol_amount
        .checked_add(lamports)
        .unwrap();
    ctx.accounts.referral_state.deposit_sol_operations = ctx
        .accounts
        .referral_state
        .deposit_sol_operations
        .checked_add(1)
        .unwrap();

    Ok(())
}
