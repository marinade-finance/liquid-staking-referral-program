use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Transfer};

use crate::constant::*;
use crate::cpi_context_instructions::{
    Deposit as MarinadeDeposit, DepositStakeAccount as MarinadeDepositStakeAccount,
    LiquidUnstake as MarinadeLiquidUnstake,
};
use crate::states::*;

//-----------------------------------------------------
#[derive(Accounts)]
// #[instruction(
//     bump: u8,
// )]
pub struct Initialize<'info> {
    // admin account
    #[account(mut, signer)]
    pub admin_account: AccountInfo<'info>,

    // global state
    // #[account(
    //     init,
    //     payer = admin_account,
    //     space = 8 + size_of::<GlobalState>(),
    //     seeds = [GLOBAL_STATE_SEED.as_ref()],
    //     bump = bump,
    // )]
    #[account(zero)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    #[account()]
    pub payment_mint: AccountInfo<'info>,

    pub system_program: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct ChangeAuthority<'info> {
    // global state
    #[account(mut, has_one = admin_account)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // current admin account (must match the one in GlobalState)
    #[account(mut, signer)]
    pub admin_account: AccountInfo<'info>,

    // new admin account
    pub new_admin_account: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
// #[instruction(
//     bump: u8,
// )]
pub struct InitReferralAccount<'info> {
    // global state
    #[account(has_one = admin_account)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account, signer
    #[account(mut, signer)]
    pub admin_account: AccountInfo<'info>,

    // partner account
    pub partner_account: AccountInfo<'info>,
    // payment token mint (normally mSOL mint)
    // #[account(address = Pubkey::from_str(MSOL_MINT_ADDRESS).unwrap())]
    #[account()]
    pub payment_mint: CpiAccount<'info, Mint>,
    // partner beneficiary mSOL ATA
    #[account()]
    pub token_partner_account: CpiAccount<'info, TokenAccount>,

    // referral state
    // #[account(
    //     init,
    //     payer = admin_account,
    //     space = 8 + 10 + size_of::<ReferralState>(),
    //     seeds = [
    //         partner_account.key().as_ref(),
    //         REFERRAL_STATE_SEED.as_ref()
    //     ],
    //     bump = bump,
    // )]
    #[account(zero)] // must be created but empty, ready to be initialized
    pub referral_state: ProgramAccount<'info, ReferralState>,

    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct UpdateReferral<'info> {
    // global state
    #[account(has_one = admin_account)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // admin account
    #[account(mut, signer)]
    pub admin_account: AccountInfo<'info>,

    // referral state
    #[account(mut)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct Deposit<'info> {
    // this part is equivalent to marinade-finance deposit instructions
    #[account(mut)]
    pub state: AccountInfo<'info>, // marinade state
    #[account(mut)]
    pub msol_mint: AccountInfo<'info>,
    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,
    #[account(mut)]
    pub reserve_pda: AccountInfo<'info>,
    #[account(mut, signer)]
    pub transfer_from: AccountInfo<'info>,
    #[account(mut)]
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,

    // accounts added are: Marinade main program ID & referral_state
    pub marinade_finance_program: AccountInfo<'info>,
    #[account(mut, constraint = !referral_state.pause)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}

impl<'info> Deposit<'info> {
    pub fn into_marinade_deposit_cpi_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, MarinadeDeposit<'info>> {
        let cpi_accounts = MarinadeDeposit {
            state: self.state.clone(),
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
    // this part is equivalent to marinade-finance deposit-stake-account instructions
    #[account(mut)]
    pub state: AccountInfo<'info>,
    #[account(mut)]
    pub validator_list: AccountInfo<'info>,
    #[account(mut)]
    pub stake_list: AccountInfo<'info>,
    #[account(mut)]
    pub stake_account: AccountInfo<'info>,
    #[account(signer)]
    pub stake_authority: AccountInfo<'info>,
    #[account(mut)]
    pub duplication_flag: AccountInfo<'info>,
    #[account(mut, signer)]
    pub rent_payer: AccountInfo<'info>,
    #[account(mut)]
    pub msol_mint: AccountInfo<'info>,
    #[account(mut)]
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub clock: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub stake_program: AccountInfo<'info>,

    // accounts added are: Marinade main program ID & referral_state
    pub marinade_finance_program: AccountInfo<'info>,
    #[account(mut, constraint = !referral_state.pause)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}

impl<'info> DepositStakeAccount<'info> {
    pub fn into_deposit_stake_account_cpi_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, MarinadeDepositStakeAccount<'info>> {
        let cpi_accounts = MarinadeDepositStakeAccount {
            state: self.state.clone(),
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
    // this part is equivalent to marinade-finance liquid-unstake instructions
    #[account(mut)]
    pub state: AccountInfo<'info>,
    #[account(mut)]
    pub msol_mint: AccountInfo<'info>,
    #[account(mut)]
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    #[account(mut)]
    pub liq_pool_msol_leg: AccountInfo<'info>,
    #[account(mut)]
    pub treasury_msol_account: AccountInfo<'info>,
    #[account(mut)]
    pub get_msol_from: AccountInfo<'info>,
    #[account(signer)]
    pub get_msol_from_authority: AccountInfo<'info>, //burn_msol_from owner or delegate_authority
    #[account(mut)]
    pub transfer_sol_to: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,

    // accounts added are: Marinade main program ID & referral_state
    pub marinade_finance_program: AccountInfo<'info>,
    #[account(mut, constraint = !referral_state.pause)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}

impl<'info> LiquidUnstake<'info> {
    pub fn into_liquid_unstake_cpi_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, MarinadeLiquidUnstake<'info>> {
        let cpi_accounts = MarinadeLiquidUnstake {
            state: self.state.clone(),
            msol_mint: self.msol_mint.clone(),
            liq_pool_sol_leg_pda: self.liq_pool_sol_leg_pda.clone(),
            liq_pool_msol_leg: self.liq_pool_msol_leg.clone(),
            treasury_msol_account: self.treasury_msol_account.clone(),
            get_msol_from: self.get_msol_from.clone(),
            get_msol_from_authority: self.get_msol_from_authority.clone(),
            transfer_sol_to: self.transfer_sol_to.clone(),
            system_program: self.system_program.clone(),
            token_program: self.token_program.clone(),
        };

        CpiContext::new(self.marinade_finance_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct TransferLiqUnstakeShares<'info> {
    // mSOL mint
    #[account(address = Pubkey::from_str(MSOL_MINT_ADDRESS).unwrap())]
    pub msol_mint: CpiAccount<'info, Mint>,

    // mSOL beneficiary account
    #[account(mut)]
    pub token_partner_account: CpiAccount<'info, TokenAccount>,

    // mSOL treasury token account
    #[account(mut)]
    pub treasury_msol_account: CpiAccount<'info, TokenAccount>,

    // treasury token account owner
    #[account(mut, signer)]
    pub treasury_account: AccountInfo<'info>,

    // referral state
    #[account(
        mut,
        constraint = !referral_state.pause,
        constraint = referral_state.liq_unstake_amount > 0,
        constraint = referral_state.token_partner_account.key() == *token_partner_account.to_account_info().key,
    )]
    pub referral_state: ProgramAccount<'info, ReferralState>,

    pub token_program: AccountInfo<'info>,
}

impl<'info> TransferLiqUnstakeShares<'info> {
    pub fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.treasury_msol_account.to_account_info().clone(),
            to: self.token_partner_account.to_account_info().clone(),
            authority: self.treasury_account.clone(),
        };

        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

//-----------------------------------------------------
#[derive(Accounts)]
pub struct DeleteAccount<'info> {
    #[account(mut, signer)]
    pub to_delete: AccountInfo<'info>,
    #[account()]
    pub beneficiary: AccountInfo<'info>,
}
