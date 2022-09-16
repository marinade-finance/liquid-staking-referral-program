use anchor_lang::prelude::*;
use marinade_finance::stake_wrapper::StakeWrapper;

use super::common::{msol_balance, transfer_msol_fee};
use crate::states::ReferralState;
use marinade_onchain_helper::{cpi_context_accounts::MarinadeDepositStakeAccount, cpi_util};

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

    // accounts added are: Marinade main program ID, referral_state, partner token account
    #[account(address = marinade_finance::ID)]
    pub marinade_finance_program: AccountInfo<'info>,
    #[account(mut, constraint = !referral_state.pause)]
    pub referral_state: ProgramAccount<'info, ReferralState>,
    #[account(mut, address = referral_state.msol_token_partner_account)]
    pub msol_token_partner_account: AccountInfo<'info>,
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

        // msol balance before call
        let msol_before = msol_balance(&self.mint_to)?;

        // prepare deposit-stake-account cpi
        let cpi_ctx = self.into_deposit_stake_account_cpi_ctx();
        let instruction_data =
            marinade_finance::instruction::DepositStakeAccount { validator_index };
        // call Marinade
        cpi_util::invoke_signed(cpi_ctx, instruction_data)?;

        // msol balance after call
        let msol_after = msol_balance(&self.mint_to)?;
        // deposit fee is transferred to referral token account
        let minted_msol = msol_after - msol_before;
        msg!(
            "minted msol {} after depositing stake account {}",
            minted_msol,
            stake_account.key()
        );
        transfer_msol_fee(
            minted_msol,
            &self.referral_state.operation_deposit_stake_account_fee,
            &self.token_program,
            &self.mint_to,
            &self.msol_token_partner_account,
            // TODO: is stake authority good authority for token stransfer?
            &self.stake_authority,
        )?;

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
