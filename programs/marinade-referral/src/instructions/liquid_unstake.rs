use anchor_lang::prelude::*;

use marinade_onchain_helper::{
    cpi_context_accounts::MarinadeLiquidUnstake, 
    cpi_util::invoke_signed,
};

use crate::states::ReferralState;

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
    #[account(address = marinade_finance::ID)]
    pub marinade_finance_program: AccountInfo<'info>,
    #[account(mut, constraint = !referral_state.pause)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
}

impl<'info> LiquidUnstake<'info> {
    pub fn process(&mut self, msol_amount: u64) -> ProgramResult {
        // accumulate treasury fees for the liquid-unstake

        // We parse manually self.state to avoid making the IDL more complex by including marinade_finance::Sate
        let marinade_state: ProgramAccount<marinade_finance::State> =
            ProgramAccount::try_from(&self.marinade_finance_program.key(), &self.state)?;
        let max_lamports = self
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
            marinade_state.check_treasury_msol_account(&self.treasury_msol_account)?;
        // cut 25% from the fee for the treasury
        let treasury_msol_cut = if is_treasury_msol_ready_for_transfer {
            marinade_state.liq_pool.treasury_cut.apply(msol_fee)
        } else {
            0
        };
        msg!("treasury_msol_cut {}", treasury_msol_cut);
        // prepare liquid-unstake cpi
        let cpi_ctx = self.into_liquid_unstake_cpi_ctx();
        let instruction_data = marinade_finance::instruction::LiquidUnstake { msol_amount };
        // call Marinade
        invoke_signed(cpi_ctx, instruction_data)?;

        // update accumulators
        self.referral_state.liq_unstake_msol_fees += treasury_msol_cut;
        self.referral_state.liq_unstake_msol_amount += msol_amount;
        self.referral_state.liq_unstake_sol_amount += user_remove_lamports;
        self.referral_state.liq_unstake_operations += 1;

        Ok(())
    }
    pub fn into_liquid_unstake_cpi_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, MarinadeLiquidUnstake<'info>> {
        let cpi_ctx = MarinadeLiquidUnstake {
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

        CpiContext::new(self.marinade_finance_program.clone(), cpi_ctx)
    }
}
