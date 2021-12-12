use anchor_lang::prelude::*;
use marinade_finance::stake_wrapper::StakeWrapper;
// use marinade_finance::{
//     instruction::DepositStakeAccount as MarinadeDepositStakeAccount, stake_wrapper::StakeWrapper,
// };
use crate::cpi_context_accounts::*;
use crate::states::ReferralState;

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
    pub fn process(&mut self, validator_index: u32) -> ProgramResult {
        // compute deposit stake account amount
        // We are parsing self.stake_account manually to avoid making the IDL more complex by including StakeWrapper
        let stake_account: CpiAccount<StakeWrapper> = CpiAccount::try_from(&self.stake_account)?;
        let delegation = stake_account.delegation().ok_or_else(|| {
            msg!(
                "Deposited stake {} must be delegated",
                stake_account.to_account_info().key
            );
            ProgramError::InvalidAccountData
        })?;
        // prepare deposit-stake-account cpi
        let cpi_ctx = self.into_deposit_stake_account_cpi_ctx();
        let instruction_data = marinade_finance::instruction::DepositStakeAccount { validator_index };
        // call Marinade
        crate::cpi_util::invoke_signed(cpi_ctx, instruction_data)?;
        // accumulate
        self.referral_state.deposit_stake_account_amount += delegation.stake;
        self.referral_state.deposit_stake_account_operations += 1;
        Ok(())
    }

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

