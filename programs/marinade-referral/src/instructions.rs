use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::{
    associated_token::Create as CreateAssociatedTokenAccount, constant::*,
    cpi_context_instructions::Deposit as MarinadeDeposit,
    cpi_context_instructions::DepositStakeAccount as MarinadeDepositStakeAccount,
    cpi_context_instructions::LiquidUnstake as MarinadeLiquidUnstake, states::*,
};

//-----------------------------------------------------
#[derive(Accounts)]
#[instruction(
    bump: u8,
)]
pub struct Initialize<'info> {
    // admin account
    #[account(mut, signer)]
    pub admin_account: AccountInfo<'info>,

    // global state
    #[account(
        init,
        payer = admin_account,
        space = 8 + size_of::<GlobalState>(),
        seeds = [GLOBAL_STATE_SEED.as_ref()],
        bump = bump,
    )]
    pub global_state: ProgramAccount<'info, GlobalState>,

    pub system_program: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct ChangeAuthority<'info> {
    // new admin account
    pub new_admin_account: AccountInfo<'info>,

    // admin account
    #[account(mut, signer)]
    pub admin_account: AccountInfo<'info>,

    // global state
    #[account(mut)]
    pub global_state: ProgramAccount<'info, GlobalState>,
}

//-----------------------------------------------------
#[derive(Accounts)]
#[instruction(
    bump: u8,
)]
pub struct CreateReferralPda<'info> {
    // mSOL mint
    pub msol_mint: CpiAccount<'info, Mint>,

    // partner account
    pub partner_account: AccountInfo<'info>,

    // partner beneficiary mSOL ATA
    #[account(mut)]
    pub beneficiary_account: AccountInfo<'info>,

    // admin account, signer
    #[account(mut, signer)]
    pub admin_account: AccountInfo<'info>,

    // referral state
    #[account(
        init,
        payer = admin_account,
        space = 8 + size_of::<ReferralState>(),
        seeds = [
            partner_account.key().as_ref(),
            REFERRAL_STATE_SEED.as_ref()
        ],
        bump = bump,
    )]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    // global state
    pub global_state: ProgramAccount<'info, GlobalState>,

    pub system_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

impl<'info> CreateReferralPda<'info> {
    pub fn into_create_associated_token_account_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, CreateAssociatedTokenAccount<'info>> {
        let cpi_accounts = CreateAssociatedTokenAccount {
            payer: self.admin_account.clone(),
            associated_token: self.beneficiary_account.clone(),
            authority: self.partner_account.clone(),
            mint: self.msol_mint.to_account_info().clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
            rent: self.rent.clone(),
        };

        CpiContext::new(self.associated_token_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct UpdateReferral<'info> {
    // admin account
    #[account(mut, signer)]
    pub admin_account: AccountInfo<'info>,

    // referral state
    #[account(mut)]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    // global state
    pub global_state: ProgramAccount<'info, GlobalState>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        // constraint = !state.pause @ ReferralError::Paused,
    )]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    #[account(signer)]
    pub transfer_from: AccountInfo<'info>,

    pub msol_mint: AccountInfo<'info>,
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,
    pub reserve_pda: AccountInfo<'info>,
    pub marinade_finance_state: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,

    // Marinade main program ID
    pub marinade_finance_program: AccountInfo<'info>,
}

impl<'info> Deposit<'info> {
    pub fn into_deposit_cpi_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MarinadeDeposit<'info>> {
        let cpi_accounts = MarinadeDeposit {
            state: self.marinade_finance_state.clone(),
            msol_mint: self.msol_mint.clone(),
            liq_pool_sol_leg_pda: self.liq_pool_sol_leg_pda.clone(),
            liq_pool_msol_leg: self.liq_pool_msol_leg.clone(),
            liq_pool_msol_leg_authority: self.liq_pool_msol_leg_authority.clone(),
            reserve_pda: self.reserve_pda.clone(),
            transfer_from: self.transfer_from.clone(),
            mint_to: self.mint_to.clone(),
            msol_mint_authority: self.msol_mint_authority.clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
        };

        CpiContext::new(self.marinade_finance_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct DepositStakeAccount<'info> {
    #[account(
        mut,
        // constraint = !state.pause @ ReferralError::Paused,
    )]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    #[account(signer)]
    pub stake_authority: AccountInfo<'info>,

    #[account(signer)]
    pub rent_payer: AccountInfo<'info>,

    pub validator_list: AccountInfo<'info>,
    pub stake_list: AccountInfo<'info>,
    pub stake_account: AccountInfo<'info>,
    pub duplication_flag: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub marinade_finance_state: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub stake_program: AccountInfo<'info>,

    // Marinade main program ID
    pub marinade_finance_program: AccountInfo<'info>,
}

impl<'info> DepositStakeAccount<'info> {
    pub fn into_deposit_stake_account_cpi_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, MarinadeDepositStakeAccount<'info>> {
        let cpi_accounts = MarinadeDepositStakeAccount {
            state: self.marinade_finance_state.clone(),
            validator_list: self.validator_list.clone(),
            stake_list: self.stake_list.clone(),
            stake_account: self.stake_account.clone(),
            stake_authority: self.stake_authority.clone(),
            duplication_flag: self.duplication_flag.clone(),
            rent_payer: self.rent_payer.clone(),
            msol_mint: self.msol_mint.clone(),
            mint_to: self.mint_to.clone(),
            msol_mint_authority: self.msol_mint_authority.clone(),
            clock: self.clock.clone(),
            rent: self.rent.clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
            stake_program: self.stake_program.clone(),
        };

        CpiContext::new(self.marinade_finance_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct LiquidUnstake<'info> {
    #[account(
        mut,
        // constraint = !state.pause @ ReferralError::Paused,
    )]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    #[account(signer)]
    pub get_msol_from_authority: AccountInfo<'info>, //burn_msol_from owner or delegate_authority
    pub msol_mint: AccountInfo<'info>,
    pub get_msol_from: AccountInfo<'info>,
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub treasury_msol_account: AccountInfo<'info>,
    pub transfer_sol_to: AccountInfo<'info>,
    pub marinade_finance_state: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,

    // Marinade main program ID
    pub marinade_finance_program: AccountInfo<'info>,
}

impl<'info> LiquidUnstake<'info> {
    pub fn into_liquid_unstake_cpi_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, MarinadeLiquidUnstake<'info>> {
        let cpi_accounts = MarinadeLiquidUnstake {
            state: self.marinade_finance_state.clone(),
            msol_mint: self.msol_mint.clone(),
            liq_pool_sol_leg_pda: self.liq_pool_sol_leg_pda.clone(),
            liq_pool_msol_leg: self.liq_pool_msol_leg.clone(),
            get_msol_from: self.get_msol_from.clone(),
            get_msol_from_authority: self.get_msol_from_authority.clone(),
            transfer_sol_to: self.transfer_sol_to.clone(),
            treasury_msol_account: self.treasury_msol_account.clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
        };

        CpiContext::new(self.marinade_finance_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct ClaimTransfer<'info> {
    pub msol_mint: CpiAccount<'info, Mint>,

    #[account(mut)]
    pub treasury_msol_account: CpiAccount<'info, TokenAccount>,

    #[account(mut)]
    pub beneficiary_account: CpiAccount<'info, TokenAccount>,

    #[account(mut, signer)]
    pub treasury_account: AccountInfo<'info>,

    #[account(mut)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}

//-----------------------------------------------------
