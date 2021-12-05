//
// Integration Test
// deposit sol & liquid unstake
//
use marinade_referral;
use marinade_referral::account_structs::*;
use marinade_referral::constant::*;

use crate::{initialize::InitializeInputWithSeeds, integration_test::*};
use marinade_finance_offchain_sdk::spl_associated_token_account::get_associated_token_address;
use marinade_finance_offchain_sdk::{
    anchor_lang::InstructionData,
    instruction_helpers::InstructionHelpers,
    marinade_finance,
    marinade_finance::{calc::proportional, liq_pool::LiqPoolHelpers, State},
    spl_token,
};

use rand::{distributions::Uniform, prelude::Distribution, CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

use solana_program::native_token::{lamports_to_sol, LAMPORTS_PER_SOL};
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::{str::FromStr, sync::Arc};
use test_env_log::test;

use crate::integration_test::test_add_remove_liquidity::*;

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

pub async fn do_deposit_sol(user: &mut TestUser, lamports: u64, test: &mut IntegrationTest) {
    //
    // get initial balances
    let user_initial_sol_balance = user.sol_balance(test).await;

    // Create a user account for msol if not exists
    let user_msol_account = user.get_or_create_msol_account(test).await;
    let user_msol_balance_before = test
        .get_token_balance_or_zero(&user_msol_account.pubkey)
        .await;

    // check lamports in reserve_pda
    let reserve_lamports_before = test
        .get_sol_balance(&State::find_reserve_address(&test.state.key).0)
        .await;
    let available_reserve_balance_before = test.state.available_reserve_balance;

    // initialize the global state
    // admin_key = AMMK9YLj8PRRG4K9DUsTNPZAZXeVbHiQJxakuVuvSKrn
    let admin_key:Keypair = Keypair::from_bytes(
        &[
          136, 60, 191, 232, 11, 20, 1, 82, 147, 185, 119, 92, 226, 212, 217, 227, 38,
          100, 72, 135, 189, 121, 32, 38, 93, 10, 41, 104, 38, 158, 171, 38, 138, 239,
          196, 48, 200, 45, 19, 235, 223, 73, 101, 62, 195, 45, 48, 246, 226, 240,
          177, 172, 213, 0, 184, 113, 158, 176, 17, 177, 2, 215, 168, 135,
        ]).unwrap();
    let mut admin = test
    .create_test_user_from_keypair("test_referral_admin_user", 200 * LAMPORTS_PER_SOL, admin_key)
    .await;

    // referral global state PDA & bump
    let (global_state_pda, global_state_bump) = Pubkey::find_program_address(
        &[GLOBAL_STATE_SEED],
        &marinade_referral::marinade_referral::ID
        );

    // test.builder
    //     .add_instruction(
    //         system_instruction::create_account(
    //             &admin.keypair.pubkey(),
    //             &global_state_pda,
    //             1_000_000, //lamports,
    //             8+std::mem::size_of::<marinade_referral::states::GlobalState>() as u64,
    //             &marinade_referral::marinade_referral::ID,
    //         ),
    //         format!("pre-create marinade-referral global state acc because banks-clients do not support creation from program {}", global_state_pda),
    //     )
    //     .unwrap();
    // test.execute().await;

    {
        let accounts = marinade_referral::accounts::Initialize {
            admin_account: admin.keypair.pubkey(),
            global_state: global_state_pda,
            system_program: system_program::ID,
        };
        let ix_data = marinade_referral::instruction::Initialize { bump:global_state_bump };
        let instruction = Instruction {
            program_id: marinade_referral::marinade_referral::ID,
            accounts: accounts.to_account_metas(None),
            data: ix_data.data(),
        };
        println!("Init global state");
        test.execute_instruction(instruction,vec!(user.keypair.clone(),admin.keypair.clone())).await;
        //}
        //let tx = Transaction::new_signed_with_payer(&[deposit_instruction], Some(&user.keypair.pubkey()),&[user.keypair.as_ref()],test.recent_blockhash().await);
        //let tx = Transaction::new_with_payer(&[instruction], Some(&user.keypair.pubkey()));
        // marinade-referral execution
        //test.execute_txn(tx, vec!(user.keypair.clone())).await;
    }
    
    let referral_state = Keypair::new();
    // initialize the referral state
    {
        let accounts = marinade_referral::accounts::Initialize {
            admin_account: Pubkey::from_str("AMMK9YLj8PRRG4K9DUsTNPZAZXeVbHiQJxakuVuvSKrn")
                .unwrap(),
            global_state: referral_state.pubkey(),
            system_program: system_program::ID,
        };
    }

    // Create a referral DepositSol instruction.
    // pub fn deposit(
    //     state: &impl Located<State>,
    //     transfer_from: Pubkey,
    //     mint_to: Pubkey,
    //     lamports: u64,
    // ) -> Instruction {
    let transfer_from = user.keypair.clone().pubkey();
    let mint_to = user_msol_account.pubkey;
    let state_key = test.state.key();

    let accounts = marinade_referral::accounts::Deposit {
        state: state_key,
        msol_mint: test.state.as_ref().msol_mint,
        liq_pool_sol_leg_pda: test.state.liq_pool_sol_leg_address(),
        liq_pool_msol_leg: test.state.as_ref().liq_pool.msol_leg,
        liq_pool_msol_leg_authority: test.state.liq_pool_msol_leg_authority(),
        reserve_pda: State::find_reserve_address(&state_key).0,
        transfer_from,
        mint_to,
        msol_mint_authority: State::find_msol_mint_authority(&state_key).0,
        system_program: system_program::ID,
        token_program: spl_token::ID,
        //----
        marinade_finance_program: marinade_finance::ID,
        referral_state: referral_state.pubkey(),
    }
    .to_account_metas(None);

    let ix_data = marinade_referral::instruction::Deposit { lamports };
    let deposit_instruction = Instruction {
        program_id: marinade_finance::ID,
        accounts,
        data: ix_data.data(),
    };
    //}
    //let tx = Transaction::new_signed_with_payer(&[deposit_instruction], Some(&user.keypair.pubkey()),&[user.keypair.as_ref()],test.recent_blockhash().await);
    let tx = Transaction::new_with_payer(&[deposit_instruction], Some(&user.keypair.pubkey()));
    // marinade-referral execution
    test.execute_txn(tx, vec!(user.keypair.clone())).await;

    // marinade-finance builder deposit
    test.builder.deposit(
        &test.state,
        user.keypair.clone(),
        user_msol_account.pubkey,
        lamports,
    );
    // execute
    test.execute().await;

    // User SOL account balance decremented.
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
    let user_msol_account = user.get_or_create_msol_account(test).await;
    let user_msol_balance_before = test
        .get_token_balance_or_zero(&user_msol_account.pubkey)
        .await;

    // Liquid unstake instruction setup
    test.builder.liquid_unstake(
        &test.state,
        user_msol_account.pubkey,
        user.keypair.clone(),
        user.keypair.pubkey(),
        msol_lamports,
    );

    let result = test.try_execute().await;
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
async fn test_deposit_sol() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        102, 221, 10, 71, 130, 126, 115, 217, 99, 44, 159, 62, 28, 73, 214, 87, 103, 93, 100, 157,
        203, 46, 9, 20, 242, 202, 225, 90, 179, 205, 107, 235,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;
    let mut user = test
        .create_test_user("test_dep_sol_user", 200 * LAMPORTS_PER_SOL)
        .await;
    do_deposit_sol(&mut user, random_amount(1, 100, &mut rng), &mut test).await;
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

    let mut alice = test
        .create_test_user("alice", 1000 * LAMPORTS_PER_SOL)
        .await;

    let alice_deposit_amount = 26 * LAMPORTS_PER_SOL;
    do_deposit_sol(&mut alice, alice_deposit_amount, &mut test).await;

    let alice_liquid_unstake_amount = 10 * LAMPORTS_PER_SOL;

    // 1st one should fail with Insufficient Liquidity in the Liquidity Pool
    const ERR_CODE_INSUFFICIENT_LIQUIDITY: u32 = 0x1199;
    match do_liquid_unstake(&mut alice, alice_liquid_unstake_amount, &mut test).await {
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
    do_liquid_unstake(&mut alice, 15 * LAMPORTS_PER_SOL, &mut test)
        .await
        .unwrap();
    Ok(())
}
