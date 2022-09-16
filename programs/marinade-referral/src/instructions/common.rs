use anchor_lang::prelude::{msg, AccountInfo, CpiContext, ProgramError};
use anchor_spl::token::{transfer, Transfer};
use marinade_finance::{calc::proportional, error::CommonError};
use solana_program::program_pack::Pack;
use std::ops::Deref;

pub fn msol_balance<'info>(mint_to: &AccountInfo<'info>) -> Result<u64, ProgramError> {
    Ok(spl_token::state::Account::unpack_from_slice(mint_to.try_borrow_data()?.deref())?.amount)
}

pub fn apply_fee(fee_basis_points: u8, amount: u64) -> Result<u64, CommonError> {
    // fee_basis_points, 10_000 = 100%
    proportional(amount, fee_basis_points as u64, 10_000u64)
}

pub fn transfer_msol_fee<'info>(
    whole_msol_amount: u64,
    fee_basis_points: u8,
    token_program: &AccountInfo<'info>,
    transfer_from: &AccountInfo<'info>,
    transfer_to: &AccountInfo<'info>,
    transfer_authority: &AccountInfo<'info>,
) -> Result<u64, ProgramError> {
    if whole_msol_amount > 0 {
        let referral_msol_amount = apply_fee(fee_basis_points, whole_msol_amount)?;
        if referral_msol_amount > 0 {
            transfer(
                CpiContext::new(
                    token_program.clone(),
                    Transfer {
                        from: transfer_from.clone(),
                        to: transfer_to.clone(),
                        authority: transfer_authority.clone(),
                    },
                ),
                referral_msol_amount,
            )?;
        }
        msg!(
            "Partner obtains {} mSOL tokens as of fee: {}bp",
            referral_msol_amount,
            fee_basis_points
        );
        Ok(referral_msol_amount)
    } else {
        msg!(
            "No mSOL {} processed at operation, no fee to be transferred",
            whole_msol_amount
        );
        Ok(0)
    }
}
