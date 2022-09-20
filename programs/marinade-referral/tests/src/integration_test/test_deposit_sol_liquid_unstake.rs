//
// Integration Test
// deposit sol & liquid unstake
//
// use marinade_referral;

use crate::integration_test::test_add_remove_liquidity::*;
use crate::{initialize::InitializeInputWithSeeds, integration_test::*};

use marinade_finance_offchain_sdk::{
    anchor_lang::InstructionData,
    marinade_finance,
    marinade_finance::{calc::proportional, liq_pool::LiqPoolHelpers, State},
    spl_token,
};

pub use spl_associated_token_account::{get_associated_token_address, ID};

use rand::{distributions::Uniform, prelude::Distribution, CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

use solana_program::native_token::{lamports_to_sol, LAMPORTS_PER_SOL};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::sync::Arc;
use test_env_log::test;

pub struct DepositSolParams {
    pub user_sol: Arc<Keypair>,
    pub user_sol_balance: u64,
    // user_msol: Pubkey,
    // user_lp: Pubkey,
    pub sol_lamports_amount: u64,
}

impl DepositSolParams {
    pub fn random<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let user_sol_balance =
            Uniform::from((1 * LAMPORTS_PER_SOL)..(10 * LAMPORTS_PER_SOL)).sample(rng);
        Self {
            user_sol: Arc::new(Keypair::generate(rng)),
            user_sol_balance,
            sol_lamports_amount: Uniform::from((LAMPORTS_PER_SOL / 2)..user_sol_balance / 4)
                .sample(rng),
        }
    }

    pub fn user_msol(&self, state: &State) -> Pubkey {
        get_associated_token_address(&self.user_sol.pubkey(), &state.msol_mint)
    }

    pub fn user_lp(&self, state: &State) -> Pubkey {
        get_associated_token_address(&self.user_sol.pubkey(), &state.liq_pool.lp_mint)
    }
}

struct TestData {
    // all balances is calculated in lamports
    pub user_msol: u64,
    pub user_msol_account: TestTokenAccount,
    pub user_sol: u64,
    pub partner_msol: u64,
    pub reserve_sol: u64,
    pub available_reserve_sol: u64,
    pub referral_state: marinade_referral::states::ReferralState,
}

impl TestData {
    async fn get(
        test: &mut IntegrationTest,
        user: &mut TestUser,
        marinade_referral_test_globals: &MarinadeReferralTestGlobals,
    ) -> Self {
        // creating a user account for msol if not exists
        let user_msol_account = user.get_or_create_msol_account_instruction(test).await;
        test.execute().await;
        let user_msol_balance = test
            .get_token_balance_or_zero(&user_msol_account.pubkey)
            .await;
        let referral_msol_balance = test
            .get_token_balance_or_zero(&marinade_referral_test_globals.msol_partner_token_pubkey)
            .await;
        let user_sol_balance = user.sol_balance(test).await;
        println!("user_sol_balance {}", user_sol_balance);
        // check lamports in reserve_pda
        let reserve_lamports = test
            .get_sol_balance(&State::find_reserve_address(&test.state.key).0)
            .await;
        let available_reserve_balance = test.state.available_reserve_balance;
        let referral_state: marinade_referral::states::ReferralState = get_account(
            test,
            marinade_referral_test_globals.partner_referral_state_pubkey,
        )
        .await;

        TestData {
            user_msol: user_msol_balance,
            user_sol: user_sol_balance,
            partner_msol: referral_msol_balance,
            reserve_sol: reserve_lamports,
            available_reserve_sol: available_reserve_balance,
            user_msol_account: user_msol_account,
            referral_state,
        }
    }
}

async fn try_deposit_execute(
    test: &mut IntegrationTest,
    user: &mut TestUser,
    marinade_instance_state: Pubkey,
    transfer_from: Pubkey,
    mint_to: Pubkey,
    partner_referral_state_pubkey: Pubkey,
    msol_token_partner_account: Pubkey,
    lamports: u64,
) -> Result<(), u32> {
    let accounts = marinade_referral::accounts::Deposit {
        state: marinade_instance_state,
        msol_mint: test.state.as_ref().msol_mint,
        liq_pool_sol_leg_pda: test.state.liq_pool_sol_leg_address(),
        liq_pool_msol_leg: test.state.as_ref().liq_pool.msol_leg,
        liq_pool_msol_leg_authority: test.state.liq_pool_msol_leg_authority(),
        reserve_pda: State::find_reserve_address(&marinade_instance_state).0,
        transfer_from,
        mint_to,
        msol_mint_authority: State::find_msol_mint_authority(&marinade_instance_state).0,
        system_program: system_program::ID,
        token_program: spl_token::ID,
        //----
        marinade_finance_program: marinade_finance::ID,
        referral_state: partner_referral_state_pubkey,
        msol_token_partner_account,
    };
    let ix_data = marinade_referral::instruction::Deposit { lamports };
    let deposit_instruction = Instruction {
        program_id: marinade_referral::marinade_referral::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data.data(),
    };
    test.try_execute_instruction(
        deposit_instruction,
        vec![test.fee_payer_signer(), user.keypair.clone()],
    )
    .await
}

async fn try_liquid_unstake(
    test: &mut IntegrationTest,
    user: &mut TestUser,
    user_msol_account: &TestTokenAccount, // msol unstaked from here
    partner_referral_state_pubkey: Pubkey,
    msol_token_partner_account: Pubkey,
    msol_lamports: u64,
) -> Result<(), u32> {
    let accounts = marinade_referral::accounts::LiquidUnstake {
        state: test.state.key(),
        get_msol_from: user_msol_account.pubkey,
        get_msol_from_authority: user.keypair.pubkey(),
        transfer_sol_to: user.keypair.pubkey(),
        treasury_msol_account: test.state.treasury_msol_account,
        msol_mint: test.state.as_ref().msol_mint,
        liq_pool_sol_leg_pda: test.state.liq_pool_sol_leg_address(),
        liq_pool_msol_leg: test.state.as_ref().liq_pool.msol_leg,
        system_program: system_program::ID,
        token_program: spl_token::ID,
        //----
        marinade_finance_program: marinade_finance::ID,
        referral_state: partner_referral_state_pubkey,
        msol_token_partner_account: msol_token_partner_account,
    };

    let ix_data = marinade_referral::instruction::LiquidUnstake {
        msol_amount: msol_lamports,
    };
    let liquid_unstake_instruction = Instruction {
        program_id: marinade_referral::marinade_referral::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data.data(),
    };
    println!("marinade-referral liquid_unstake");
    test.try_execute_instruction(
        liquid_unstake_instruction,
        vec![test.fee_payer_signer(), user.keypair.clone()],
    )
    .await
}

async fn do_deposit_sol(
    user: &mut TestUser,
    lamports: u64,
    test: &mut IntegrationTest,
    marinade_referral_test_globals: &MarinadeReferralTestGlobals,
    operation_fee_bps: u8,
) -> Result<(), u32> {
    let data_before = TestData::get(test, user, marinade_referral_test_globals).await;

    // Set-up fee for deposit sol operation
    update_operation_fees(
        test,
        marinade_referral_test_globals.global_state_pubkey,
        &marinade_referral_test_globals.admin_key,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        Some(operation_fee_bps),
        None,
        None,
        None,
    )
    .await?;

    // -----------------------------------------
    // Create a referral DepositSol instruction.
    // -----------------------------------------
    try_deposit_execute(
        test,
        user,
        test.state.key(),
        user.keypair.clone().pubkey(),        // transfer_from
        data_before.user_msol_account.pubkey, // mint_to
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
        lamports,
    )
    .await?;

    // // marinade-finance builder deposit
    // commented: Direct call to marinade
    // test.builder.deposit(
    //     &test.state,
    //     user.keypair.clone(),
    //     user_msol_account.pubkey,
    //     lamports,
    // );
    // // execute
    // test.execute().await;

    let data_after = TestData::get(test, user, marinade_referral_test_globals).await;

    let operation_fee_lamports = (lamports as u64 * operation_fee_bps as u64 / 10_000_u64) as u64;
    println!(
        "Basis points fee: {}, deposited lamports: {}, referral fee for deposit operation: {}",
        operation_fee_bps, lamports, operation_fee_lamports
    );

    // Check User SOL account balance decremented.
    assert_eq!(data_after.user_sol, data_before.user_sol - lamports);
    // User's mSOL account credited.
    // TODO: use test.state.msol_price & then compute correct msol received result
    // for now, since mSOL price=1 we expect the same amount as deposited lamports
    assert_eq!(
        data_after.user_msol,
        data_before.user_msol + lamports - operation_fee_lamports
    );
    // check lamports in reserve_pda
    assert_eq!(data_after.reserve_sol, data_before.reserve_sol + lamports);
    // check also computed state field state.available_reserve_balance
    assert_eq!(
        data_after.available_reserve_sol,
        data_before.available_reserve_sol + lamports
    );
    // check the partner account was credited
    assert_eq!(
        data_after.partner_msol,
        data_before.partner_msol + operation_fee_lamports
    );
    assert_eq!(
        data_before.referral_state.accum_deposit_sol_fees + operation_fee_lamports,
        data_after.referral_state.accum_deposit_sol_fees,
        "Deposit sol operation accumulator fee does not increased by exepected amount"
    );
    Ok(())
}

pub async fn do_liquid_unstake(
    user: &mut TestUser,
    msol_lamports: u64,
    test: &mut IntegrationTest,
    marinade_referral_test_globals: &MarinadeReferralTestGlobals,
    operaration_fee_bps: u8,
) -> Result<(), u32> {
    println!(
        "--- do_liquid_unstake {} mSOL ----------",
        lamports_to_sol(msol_lamports)
    );
    // let user_sol_balance_before = test.show_user_balance(&user, "before").await;

    // get sol_leg address
    let sol_leg_address = test.state.liq_pool_sol_leg_address();
    let liquidity_lamports = test.get_sol_balance(&sol_leg_address).await;
    println!("--- liquidity {} ", lamports_to_sol(liquidity_lamports));

    // // Create a user account for msol if not exists
    // let user_msol_account = user.get_or_create_msol_account_instruction(test).await;
    // let user_msol_balance_before = test
    //     .get_token_balance_or_zero(&user_msol_account.pubkey)
    //     .await;

    // let partner_msol_balance_before = test
    //     .get_token_balance(&marinade_referral_test_globals.msol_partner_token_pubkey)
    //     .await;

    // let referral_state_before: marinade_referral::states::ReferralState = get_account(
    //     test,
    //     marinade_referral_test_globals.partner_referral_state_pubkey,
    // )
    // .await;
    let data_before = TestData::get(test, user, marinade_referral_test_globals).await;
    assert_eq!(
        operaration_fee_bps, data_before.referral_state.operation_liquid_unstake_fee,
        "Operation fee was expected setup to diffeent value than found in account."
    );
    let operation_fee_lamports =
        msol_lamports * data_before.referral_state.operation_liquid_unstake_fee as u64 / 10_000;
    let msol_lamports_fee_deducted = msol_lamports - operation_fee_lamports;

    // -----------------------------------------
    // Create a referral LiquidUnstake instruction.
    // -----------------------------------------
    let result = try_liquid_unstake(
        test,
        user,
        &data_before.user_msol_account, // msol unstaked from here
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
        msol_lamports,
    )
    .await;

    // COMMENTED - Direct call to marinade-finance
    // test.builder.liquid_unstake(
    //     &test.state,
    //     user_msol_account.pubkey,
    //     user.keypair.clone(),
    //     user.keypair.pubkey(),
    //     msol_lamports,
    // );
    // let result = test.try_execute().await;

    if result.is_err() {
        println!("-- do_liquid_unstake result: {:x?}", result);
        return result;
    }

    // compute liq unstake fee
    assert!(msol_lamports_fee_deducted < liquidity_lamports);
    // fee is computed on the amount *after* the user swaps
    let fee_basis_points =
        if liquidity_lamports - msol_lamports > test.state.liq_pool.lp_liquidity_target {
            test.state.liq_pool.lp_min_fee.basis_points
        } else {
            // fee is max_fee - proportional: delta * liquidity_after / liquidity_target
            // the fee is on a linear curve from max_fee to min_fee, where max_fee is on 0 and min_fee on liquidity_target
            test.state.liq_pool.lp_max_fee.basis_points
                - proportional(
                    test.state.liq_pool.delta() as u64,
                    liquidity_lamports - msol_lamports_fee_deducted,
                    test.state.liq_pool.lp_liquidity_target,
                )
                .unwrap() as u32
        };

    println!("--------------------------");
    println!(
        "liquidity: {}, unstake fee: {}%, liquidity target: {}, operation fee: {}bps",
        lamports_to_sol(liquidity_lamports),
        fee_basis_points as f64 / 100.0,
        lamports_to_sol(test.state.liq_pool.lp_liquidity_target),
        operaration_fee_bps,
    );

    // liquid unstake fee
    let liquid_unstake_fee = marinade_finance::Fee {
        basis_points: fee_basis_points,
    };
    // compute fee in msol
    let msol_fee = liquid_unstake_fee.apply(msol_lamports_fee_deducted);
    // assuming is_treasury_msol_ready_for_transfer is always true
    let treasury_msol_cut = test.state.liq_pool.treasury_cut.apply(msol_fee);

    let data_after = TestData::get(test, user, marinade_referral_test_globals).await;

    // Check treasury_msol_cut == referral_state.liq_unstake_msol_fees
    assert_eq!(
        data_after.referral_state.liq_unstake_msol_fees,
        treasury_msol_cut
    );
    assert_eq!(
        data_after.referral_state.liq_unstake_msol_amount,
        msol_lamports_fee_deducted
    );

    // msol_amount in lamports
    let user_remove_lamports = test
        .state
        .calc_lamports_from_msol_amount(msol_lamports_fee_deducted)
        .unwrap();
    assert_eq!(
        data_after.referral_state.liq_unstake_sol_amount,
        user_remove_lamports
    );

    // Check post-conditions.
    let user_sol_balance_after = user.sol_balance(test).await;
    assert_eq!(
        user_sol_balance_after,
        data_before.user_sol + msol_lamports_fee_deducted
            - proportional(msol_lamports_fee_deducted, fee_basis_points as u64, 10_000).unwrap()
    );
    // user reduced number token account for all mSOLs but fee was deduced from native lamports transferred
    assert_eq!(data_after.user_msol, data_before.user_msol - msol_lamports);
    // Check post-conditions of operation fees
    assert_eq!(
        data_after.partner_msol - data_before.partner_msol,
        operation_fee_lamports,
        "Partner is expected to receive mSOL in the amount of the operation fee"
    );
    assert_eq!(
        data_before.referral_state.accum_liquid_unstake_fee + operation_fee_lamports,
        data_after.referral_state.accum_liquid_unstake_fee,
        "Liquid unstake operation accumulator fee does not increased by exepected amount"
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_deposit_sol_no_fees() -> anyhow::Result<()> {
    let (mut test, marinade_referral_test_globals, mut rng) = IntegrationTest::init_test().await?;
    let mut user = test
        .create_test_user("test_dep_sol_user", 200 * LAMPORTS_PER_SOL)
        .await;

    do_deposit_sol(
        &mut user,
        random_amount(1, 100, &mut rng),
        &mut test,
        &marinade_referral_test_globals,
        0,
    )
    .await
    .unwrap();
    Ok(())
}

#[test(tokio::test)]
async fn test_deposit_sol_operation_fee() -> anyhow::Result<()> {
    let (mut test, marinade_referral_test_globals, mut rng) = IntegrationTest::init_test().await?;
    let mut user = test
        .create_test_user("test_dep_sol_user", 200 * LAMPORTS_PER_SOL)
        .await;

    do_deposit_sol(
        &mut user,
        random_amount(1, 100, &mut rng),
        &mut test,
        &marinade_referral_test_globals,
        27,
    )
    .await
    .unwrap();

    Ok(())
}

#[test(tokio::test)]
async fn test_deposit_sol_wrong_referral() -> anyhow::Result<()> {
    let (mut test, marinade_referral_test_globals, _) = IntegrationTest::init_test().await?;
    let mut user = test
        .create_test_user("test_dep_sol_user", 200 * LAMPORTS_PER_SOL)
        .await;
    let user_msol_account = user.get_or_create_msol_account_instruction(&mut test).await;

    let marinade_instance_state = test.state.key();
    let depositor = user.keypair.clone().pubkey();
    let deposit_result = try_deposit_execute(
        &mut test,
        &mut user,
        marinade_instance_state,
        depositor,                // transfer_from
        user_msol_account.pubkey, // mint_to
        marinade_referral_test_globals.partner_referral_state_pubkey,
        user_msol_account.pubkey,
        22,
    )
    .await;
    match deposit_result {
        Ok(_) => panic!("Expected error happens when user want to be a referral"),
        Err(number) => {
            // anchor 0.14.0 : error 152 : ConstraintAddress
            assert_eq!(
                152, number,
                "Expected anchor error 'An address constraint was violated'"
            );
        }
    }
    Ok(())
}

#[test(tokio::test)]
async fn test_liquid_unstake() -> anyhow::Result<()> {
    let (mut test, marinade_referral_test_globals, _) = IntegrationTest::init_test().await?;

    let mut alice = test
        .create_test_user("alice", 1000 * LAMPORTS_PER_SOL)
        .await;

    let alice_deposit_amount = 26 * LAMPORTS_PER_SOL;
    do_deposit_sol(
        &mut alice,
        alice_deposit_amount,
        &mut test,
        &marinade_referral_test_globals,
        0,
    )
    .await
    .unwrap();

    let alice_liquid_unstake_amount = 10 * LAMPORTS_PER_SOL;

    // 1st one should fail with Insufficient Liquidity in the Liquidity Pool
    const ERR_CODE_INSUFFICIENT_LIQUIDITY: u32 = 0x1199;
    match do_liquid_unstake(
        &mut alice,
        alice_liquid_unstake_amount,
        &mut test,
        &marinade_referral_test_globals,
        0,
    )
    .await
    {
        Ok(()) => debug_assert!(false, "expected err got Ok"),
        Err(ERR_CODE_INSUFFICIENT_LIQUIDITY) => println!(
            "(expected tx failure 0x{:x})",
            ERR_CODE_INSUFFICIENT_LIQUIDITY
        ),
        Err(x) => debug_assert!(
            false,
            "expected err(ERR_CODE_INSUFFICIENT_LIQUIDITY) got 0x{:x}",
            x
        ),
    }

    // set referral fees, so liquidity unstake operation is charged with referral mSOL fee 30bp
    let operation_fee = 30;
    update_operation_fees(
        &mut test,
        marinade_referral_test_globals.global_state_pubkey,
        &marinade_referral_test_globals.admin_key,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        Some(0),
        Some(0),
        Some(operation_fee),
        Some(0),
    )
    .await
    .unwrap();

    // add liquidity :: bob adds liquidity
    let mut bob = test
        .create_test_user("bob", 50_000 * LAMPORTS_PER_SOL)
        .await;
    do_add_liquidity(&mut bob, 25 * LAMPORTS_PER_SOL, &mut test)
        .await
        .unwrap();

    // 2nd should work ok
    do_liquid_unstake(
        &mut alice,
        15 * LAMPORTS_PER_SOL,
        &mut test,
        &marinade_referral_test_globals,
        operation_fee,
    )
    .await
    .unwrap();
    Ok(())
}

#[test(tokio::test)]
async fn test_liquid_unstake_wrong_referral() -> anyhow::Result<()> {
    let (mut test, marinade_referral_test_globals, _) = IntegrationTest::init_test().await?;

    let mut user = test
        .create_test_user("test_dep_sol_user", 200 * LAMPORTS_PER_SOL)
        .await;
    let user_msol_account = user.get_or_create_msol_account_instruction(&mut test).await;

    let unstake_result = try_liquid_unstake(
        &mut test,
        &mut user,
        &user_msol_account, // msol unstaked from here
        marinade_referral_test_globals.partner_referral_state_pubkey,
        user_msol_account.pubkey,
        22,
    )
    .await;
    match unstake_result {
        Ok(_) => panic!("Expected error happens when user want to be a referral"),
        Err(number) => {
            // anchor 0.14.0 : error 152 : ConstraintAddress
            assert_eq!(
                152, number,
                "Expected anchor error 'An address constraint was violated'"
            );
        }
    }
    Ok(())
}
