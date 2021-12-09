use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use marinade_finance::{
    instruction::DepositStakeAccount as MarinadeDepositStakeAccount, stake_wrapper::StakeWrapper,
    State as MarinadeSate,
};

use crate::account_structs::*;

pub fn process_deposit_stake_account(
    ctx: Context<DepositStakeAccount>,
    validator_index: u32,
) -> ProgramResult {
    msg!("enter process_deposit_stake_account");
    // deposit-stake-account cpi
    let cpi_ctx = ctx.accounts.into_deposit_stake_account_cpi_ctx();
    let cpi_accounts = cpi_ctx.to_account_metas(None);
    let data = MarinadeDepositStakeAccount { validator_index };
    let ix = Instruction {
        program_id: *cpi_ctx.program.key,
        accounts: cpi_accounts,
        data: data.data(),
    };
    msg!("call marinade deposit_stake_account");
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            cpi_ctx.accounts.state,
            cpi_ctx.accounts.validator_list,
            cpi_ctx.accounts.stake_list,
            cpi_ctx.accounts.stake_account,
            cpi_ctx.accounts.stake_authority,
            cpi_ctx.accounts.duplication_flag,
            cpi_ctx.accounts.rent_payer,
            cpi_ctx.accounts.msol_mint,
            cpi_ctx.accounts.mint_to,
            cpi_ctx.accounts.msol_mint_authority,
            cpi_ctx.accounts.system_program,
            cpi_ctx.accounts.token_program,
            cpi_ctx.accounts.stake_program,
            cpi_ctx.accounts.clock,
            cpi_ctx.accounts.rent,
            //
            ctx.accounts.marinade_finance_program.clone(),
        ],
        cpi_ctx.signer_seeds,
    )?;

    // compute msol_to_mint
    let marinade_state: ProgramAccount<MarinadeSate> = ProgramAccount::try_from(
        &ctx.accounts.marinade_finance_program.key(),
        &ctx.accounts.state,
    )?;
    let stake_account: CpiAccount<StakeWrapper> =
        CpiAccount::try_from(&ctx.accounts.stake_account)?;
    let delegation = stake_account.delegation().ok_or_else(|| {
        msg!(
            "Deposited stake {} must be delegated",
            stake_account.to_account_info().key
        );
        ProgramError::InvalidAccountData
    })?;
    let msol_to_mint = marinade_state.calc_msol_from_lamports(delegation.stake)?;

    msg!("deposit_stake_account accumulators");
    ctx.accounts.referral_state.deposit_stake_account_amount = ctx
        .accounts
        .referral_state
        .deposit_stake_account_amount
        .wrapping_add(msol_to_mint);
    ctx.accounts.referral_state.deposit_stake_account_operations = ctx
        .accounts
        .referral_state
        .deposit_stake_account_operations
        .wrapping_add(1);

    Ok(())
}
