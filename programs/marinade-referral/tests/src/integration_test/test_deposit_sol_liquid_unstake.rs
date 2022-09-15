//
// Integration Test
// deposit sol & liquid unstake
//
// use marinade_referral;

use crate::{initialize::InitializeInputWithSeeds, integration_test::*};
use crate::integration_test::test_add_remove_liquidity::*;

use marinade_finance_offchain_sdk::{
    anchor_lang::InstructionData,
    marinade_finance,
    marinade_finance::{calc::proportional, liq_pool::LiqPoolHelpers, State},
    spl_token,
};

pub use spl_associated_token_account::{get_associated_token_address, ID};

use rand_chacha::ChaChaRng;
use rand::{distributions::Uniform, prelude::Distribution, CryptoRng, RngCore, SeedableRng};

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


async fn deposit_execute(
    test: &mut IntegrationTest,
    user: &mut TestUser,
    marinade_instance_state: Pubkey,
    transfer_from: Pubkey,
    mint_to: Pubkey,
    partner_referral_state_pubkey: Pubkey,
    msol_token_partner_account: Pubkey,
    lamports: u64,
) {
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
    //}
    test.execute_instruction(deposit_instruction, vec![test.fee_payer_signer(), user.keypair.clone()]).await;
}

pub async fn do_deposit_sol_no_fee(
    user: &mut TestUser,
    lamports: u64,
    test: &mut IntegrationTest,
    marinade_referral_test_globals: &MarinadeReferralTestGlobals,
) {
    // Create a user account for msol if not exists
    let user_msol_account = user.get_or_create_msol_account_instruction(test).await;
    test.execute().await;

    // Set operation fees to zero to simplify balance calculations
    update_referral_execute(
        test,
        marinade_referral_test_globals.global_state_pubkey,
        &marinade_referral_test_globals.admin_key,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.partner.keypair.pubkey(),
        marinade_referral_test_globals.msol_partner_token_pubkey,
        false,
        Some(0),
        Some(0),
        Some(0),
        Some(0),
    ).await.unwrap();

    let user_msol_balance_before = test
        .get_token_balance_or_zero(&user_msol_account.pubkey)
        .await;

    // check lamports in reserve_pda
    let reserve_lamports_before = test
        .get_sol_balance(&State::find_reserve_address(&test.state.key).0)
        .await;
    let available_reserve_balance_before = test.state.available_reserve_balance;

    // save user balance pre-deposit
    let user_initial_sol_balance = user.sol_balance(test).await;
    println!("user_initial_sol_balance {}", user_initial_sol_balance);

    // -----------------------------------------
    // Create a referral DepositSol instruction.
    // -----------------------------------------
    deposit_execute(
        test,
        user,
        test.state.key(),
        user.keypair.clone().pubkey(),  // transfer_from
        user_msol_account.pubkey,  // mint_to
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
        lamports
    ).await;

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

    // Check User SOL account balance decremented.
    let user_sol_balance_after = user.sol_balance(test).await;
    assert_eq!(user_sol_balance_after, user_initial_sol_balance - lamports);

    // User's mSOL account credited.
    let user_msol_balance_after = test.get_token_balance(&user_msol_account.pubkey).await;
    // TODO: use test.state.msol_price & then compute correct msol received result
    // for now, since mSOL price=1 we expect the same amount as deposited lamports
    assert_eq!(user_msol_balance_after, user_msol_balance_before + lamports);

    // check lamports in reserve_pda
    let reserve_lamports_after_stake = test
        .get_sol_balance(&State::find_reserve_address(&test.state.key).0)
        .await;
    assert_eq!(
        reserve_lamports_after_stake,
        reserve_lamports_before + lamports
    );
    // check also computed state field state.available_reserve_balance
    assert_eq!(
        test.state.available_reserve_balance,
        available_reserve_balance_before + lamports
    );
}

pub async fn do_liquid_unstake(
    user: &mut TestUser,
    msol_lamports: u64,
    test: &mut IntegrationTest,
    marinade_referral_test_globals: &MarinadeReferralTestGlobals,
) -> Result<(), u32> {
    println!(
        "--- do_liquid_unstake {} mSOL ----------",
        lamports_to_sol(msol_lamports)
    );
    let user_sol_balance_before = test.show_user_balance(&user, "before").await;

    // get sol_leg address
    let sol_leg_address = test.state.liq_pool_sol_leg_address();
    let liquidity_lamports = test.get_sol_balance(&sol_leg_address).await;
    println!("--- liquidity {} ", lamports_to_sol(liquidity_lamports));

    // Create a user account for msol if not exists
    let user_msol_account = user.get_or_create_msol_account_instruction(test).await;
    let user_msol_balance_before = test
        .get_token_balance_or_zero(&user_msol_account.pubkey)
        .await;

    // Liquid unstake instruction setup

    // -----------------------------------------
    // Create a referral LiquidUnstake instruction.
    // -----------------------------------------
    let state_key = test.state.key();
    let user_key = user.keypair.clone().pubkey();
    let get_msol_from = user_msol_account.pubkey;

    let accounts = marinade_referral::accounts::LiquidUnstake {
        state: state_key,
        get_msol_from,
        get_msol_from_authority: user_key,
        transfer_sol_to: user_key,
        treasury_msol_account: test.state.treasury_msol_account,
        msol_mint: test.state.as_ref().msol_mint,
        liq_pool_sol_leg_pda: test.state.liq_pool_sol_leg_address(),
        liq_pool_msol_leg: test.state.as_ref().liq_pool.msol_leg,
        system_program: system_program::ID,
        token_program: spl_token::ID,
        //----
        marinade_finance_program: marinade_finance::ID,
        referral_state: marinade_referral_test_globals.partner_referral_state_pubkey,
    }
    .to_account_metas(None);

    let ix_data = marinade_referral::instruction::LiquidUnstake {
        msol_amount: msol_lamports,
    };
    let liquid_unstake_instruction = Instruction {
        program_id: marinade_referral::marinade_referral::ID,
        accounts,
        data: ix_data.data(),
    };
    let tx = Transaction::new_with_payer(&[liquid_unstake_instruction], Some(&test.fee_payer()));
    // marinade-referral liquid_unstake
    println!("marinade-referral liquid_unstake");
    let result = test
        .try_execute_txn(tx, vec![test.fee_payer_signer(), user.keypair.clone()])
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
    assert!(msol_lamports < liquidity_lamports);
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
                    liquidity_lamports - msol_lamports,
                    test.state.liq_pool.lp_liquidity_target,
                )
                .unwrap() as u32
        };

    println!("--------------------------");
    println!(
        "liquidity: {}, unstake fee: {}%, liquidity target: {}",
        lamports_to_sol(liquidity_lamports),
        fee_basis_points as f64 / 100.0,
        lamports_to_sol(test.state.liq_pool.lp_liquidity_target)
    );

    // liquid unstake fee
    let liquid_unstake_fee = marinade_finance::Fee {
        basis_points: fee_basis_points,
    };
    // compute fee in msol
    let msol_fee = liquid_unstake_fee.apply(msol_lamports);
    // assuming is_treasury_msol_ready_for_transfer is always true
    let treasury_msol_cut = test.state.liq_pool.treasury_cut.apply(msol_fee);

    // read MARINADE-FINANCE-REFERRAL-PROGRAM state
    let referral_state: marinade_referral::states::ReferralState =
        get_account(test, marinade_referral_test_globals.partner_referral_state_pubkey).await;

    // Check treasury_msol_cut == referral_state.liq_unstake_msol_fees
    assert_eq!(referral_state.liq_unstake_msol_fees, treasury_msol_cut);
    assert_eq!(referral_state.liq_unstake_msol_amount, msol_lamports);

    // msol_amount in lamports
    let user_remove_lamports = test
        .state
        .calc_lamports_from_msol_amount(msol_lamports)
        .unwrap();
    assert_eq!(referral_state.liq_unstake_sol_amount, user_remove_lamports);

    // Check post-conditions.
    let user_sol_balance_after = user.sol_balance(test).await;
    assert_eq!(
        user_sol_balance_after,
        user_sol_balance_before + msol_lamports
            - proportional(msol_lamports, fee_basis_points as u64, 10_000).unwrap()
    );

    let user_msol_balance_after = test.show_token_balance(&user_msol_account, "after").await;
    assert_eq!(
        user_msol_balance_after,
        user_msol_balance_before - msol_lamports
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_deposit_sol_no_fees() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        102, 221, 10, 71, 130, 126, 115, 217, 99, 44, 159, 62, 28, 73, 214, 87, 103, 93, 100, 157,
        203, 46, 9, 20, 242, 202, 225, 90, 179, 205, 107, 235,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;
    let mut user = test
        .create_test_user("test_dep_sol_user", 200 * LAMPORTS_PER_SOL)
        .await;

    let marinade_referral_test_globals = init_marinade_referral_test_globals(&mut test).await;

    do_deposit_sol_no_fee(
        &mut user,
        random_amount(1, 100, &mut rng),
        &mut test,
        &marinade_referral_test_globals,
    )
    .await;
    Ok(())
}

#[test(tokio::test)]
async fn test_deposit_sol_operation_fee() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        102, 221, 10, 71, 130, 126, 115, 217, 99, 44, 159, 62, 28, 73, 214, 87, 103, 93, 100, 157,
        203, 46, 9, 20, 242, 202, 225, 90, 179, 205, 107, 235,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;
    let mut user = test
        .create_test_user("test_dep_sol_user", 200 * LAMPORTS_PER_SOL)
        .await;
    let marinade_referral_test_globals = init_marinade_referral_test_globals(&mut test).await;

    let user_msol_account = user.get_or_create_msol_account_instruction(&mut test).await;
    test.execute().await;
    let user_msol_balance_before = test
        .get_token_balance_or_zero(&user_msol_account.pubkey)
        .await;
    let referral_msol_balance_before = test
        .get_token_balance_or_zero(&marinade_referral_test_globals.msol_partner_token_pubkey)
        .await;
    let user_initial_sol_balance = user.sol_balance(&mut test).await;
    println!("user_initial_sol_balance {}", user_initial_sol_balance);

    // configuring fee for referrals at 50 basis points
    let lamports = random_amount(1, 100, &mut rng);
    let basis_points_fee = 50;
    update_referral_execute(
        &mut test,
        marinade_referral_test_globals.global_state_pubkey,
        &marinade_referral_test_globals.admin_key,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.partner.keypair.pubkey(),
        marinade_referral_test_globals.msol_partner_token_pubkey,
        false,
        Some(basis_points_fee),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    let transfer_from = user.keypair.clone().pubkey();
    let mint_to = user_msol_account.pubkey;
    let marinade_instance_state = test.state.key();
    deposit_execute(
        &mut test,
        &mut user,
        marinade_instance_state,
        transfer_from,
        mint_to,
        marinade_referral_test_globals.partner_referral_state_pubkey,
        marinade_referral_test_globals.msol_partner_token_pubkey,
        lamports,
    )
    .await;

    let referral_fee = (lamports as u64 * basis_points_fee as u64 / 10_000_u64) as u64;
    assert!(
        referral_fee > 0 && referral_fee <= lamports,
        "Refferal fee cannot be zero and bigger than 100%"
    );
    println!(
        "Basis points fee: {}, deposited lamports: {}, referral fee for deposit operation: {}",
        basis_points_fee, lamports, referral_fee
    );
    // user SOL and mSOL credit after deposit opration
    let user_sol_balance_after = user.sol_balance(&mut test).await;
    assert_eq!(user_sol_balance_after, user_initial_sol_balance - lamports);
    let user_msol_balance_after = test.get_token_balance(&user_msol_account.pubkey).await;
    assert_eq!(
        user_msol_balance_after,
        user_msol_balance_before + lamports - referral_fee
    );
    let referral_msol_balance_after = test
        .get_token_balance_or_zero(&marinade_referral_test_globals.msol_partner_token_pubkey)
        .await;
    assert_eq!(
        referral_msol_balance_after,
        referral_msol_balance_before + referral_fee
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_liquid_unstake() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        133, 212, 66, 197, 183, 220, 98, 25, 113, 166, 123, 89, 163, 64, 63, 122, 141, 42, 124, 91,
        169, 181, 200, 41, 48, 38, 37, 39, 213, 137, 222, 165,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;

    let marinade_referral_test_globals = init_marinade_referral_test_globals(&mut test).await;

    let mut alice = test
        .create_test_user("alice", 1000 * LAMPORTS_PER_SOL)
        .await;

    let alice_deposit_amount = 26 * LAMPORTS_PER_SOL;
    do_deposit_sol_no_fee(
        &mut alice,
        alice_deposit_amount,
        &mut test,
        &marinade_referral_test_globals,
    )
    .await;

    let alice_liquid_unstake_amount = 10 * LAMPORTS_PER_SOL;

    // 1st one should fail with Insufficient Liquidity in the Liquidity Pool
    const ERR_CODE_INSUFFICIENT_LIQUIDITY: u32 = 0x1199;
    match do_liquid_unstake(
        &mut alice,
        alice_liquid_unstake_amount,
        &mut test,
        &marinade_referral_test_globals,
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

    // add liquidity
    // bob adds liquidity
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
    )
    .await
    .unwrap();
    Ok(())
}
