use anchor_lang::prelude::*;

// Marinade Instructions structs but without parsed data, all fields are AccountInfo

#[derive(Accounts)]
pub struct MarinadeDeposit<'info> {
    pub state: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub liq_pool_msol_leg_authority: AccountInfo<'info>,
    pub reserve_pda: AccountInfo<'info>,
    pub transfer_from: AccountInfo<'info>,
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MarinadeDepositStakeAccount<'info> {
    pub state: AccountInfo<'info>,
    pub validator_list: AccountInfo<'info>,
    pub stake_list: AccountInfo<'info>,
    pub stake_account: AccountInfo<'info>,
    pub stake_authority: AccountInfo<'info>,
    pub duplication_flag: AccountInfo<'info>,
    pub rent_payer: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub mint_to: AccountInfo<'info>,
    pub msol_mint_authority: AccountInfo<'info>,
    pub clock: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub stake_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MarinadeLiquidUnstake<'info> {
    pub state: AccountInfo<'info>,
    pub msol_mint: AccountInfo<'info>,
    pub liq_pool_sol_leg_pda: AccountInfo<'info>,
    pub liq_pool_msol_leg: AccountInfo<'info>,
    pub treasury_msol_account: AccountInfo<'info>,
    pub get_msol_from: AccountInfo<'info>,
    pub get_msol_from_authority: AccountInfo<'info>,
    pub transfer_sol_to: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}
