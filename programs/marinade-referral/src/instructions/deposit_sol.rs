use anchor_lang::prelude::*;

use marinade_onchain_helper::{cpi_context_accounts::MarinadeDeposit, cpi_util};

use crate::states::ReferralState;

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
    #[account(address = marinade_finance::ID)]
    pub marinade_finance_program: AccountInfo<'info>,
    #[account(mut, constraint = !referral_state.pause)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}

impl<'info> Deposit<'info> {
    pub fn process(&mut self, lamports: u64) -> ProgramResult {
        msg!("enter Deposit::process {}", lamports);
        let cpi_ctx = self.into_marinade_deposit_cpi_ctx();
        let data = marinade_finance::instruction::Deposit { lamports };
        // call Marinade
        cpi_util::invoke_signed(cpi_ctx, data)?;
        // update accumulators
        self.referral_state.deposit_sol_amount += lamports;
        self.referral_state.deposit_sol_operations += 1;
        Ok(())
    }

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
