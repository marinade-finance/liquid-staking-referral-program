use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
pub use marinade_finance::instruction::Deposit as MarinadeDeposit;

use crate::instructions::*;

pub fn process_deposit(ctx: Context<Deposit>, lamports: u64) -> ProgramResult {
    ctx.accounts.state.deposit_sol_amount =
        ctx.accounts.state.deposit_sol_amount.wrapping_add(lamports);
    ctx.accounts.state.deposit_sol_operations =
        ctx.accounts.state.deposit_sol_operations.wrapping_add(1);

    // Deposit SOL cpi
    let cpi_ctx = ctx.accounts.into_deposit_sol_cpi_ctx();
    let cpi_accounts = cpi_ctx.to_account_metas(None);
    let data = MarinadeDeposit { lamports };
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
            cpi_ctx.accounts.liq_pool_msol_leg_authority,
            cpi_ctx.accounts.reserve_pda,
            cpi_ctx.accounts.transfer_from,
            cpi_ctx.accounts.mint_to,
            cpi_ctx.accounts.msol_mint_authority,
            cpi_ctx.accounts.system_program,
            cpi_ctx.accounts.token_program,
        ],
        cpi_ctx.signer_seeds,
    )?;

    Ok(())
}
