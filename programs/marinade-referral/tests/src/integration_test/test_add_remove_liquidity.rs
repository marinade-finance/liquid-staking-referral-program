//
// Integration Test
// add & remove liquidity
//
use std::sync::Arc;
use crate::{initialize::InitializeInputWithSeeds, integration_test::*};

use marinade_finance_offchain_sdk::spl_associated_token_account::get_associated_token_address;
use marinade_finance_offchain_sdk::{instruction_helpers::InstructionHelpers, marinade_finance::State};

use rand::{distributions::Uniform, prelude::Distribution, CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use test_env_log::test;

pub struct AddLiquidityParams {
    pub user_sol: Arc<Keypair>,
    pub user_sol_balance: u64,
    // user_msol: Pubkey,
    // user_lp: Pubkey,
    pub added_liquidity: u64,
}

impl AddLiquidityParams {
    pub fn random<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        let user_sol_balance =
            Uniform::from((1 * LAMPORTS_PER_SOL)..(10 * LAMPORTS_PER_SOL)).sample(rng);
        Self {
            user_sol: Arc::new(Keypair::generate(rng)),
            user_sol_balance,
            added_liquidity: Uniform::from((LAMPORTS_PER_SOL / 2)..user_sol_balance).sample(rng),
        }
    }

    pub fn user_msol(&self, state: &State) -> Pubkey {
        get_associated_token_address(&self.user_sol.pubkey(), &state.msol_mint)
    }

    pub fn user_lp(&self, state: &State) -> Pubkey {
        get_associated_token_address(&self.user_sol.pubkey(), &state.liq_pool.lp_mint)
    }
}

pub async fn do_add_liquidity(
    user: &mut TestUser,
    lamports: u64,
    test: &mut IntegrationTest,
) -> Result<(), u32> {
    //
    let user_sol_balance_before = user.sol_balance(test).await;

    // Create a user account for msol if not exists
    let user_msol_account = user.get_or_create_msol_account_instruction(test).await;
    let user_msol_balance_before = test
        .get_token_balance_or_zero(&user_msol_account.pubkey)
        .await;

    // Create a user account for LP tokens if not exists
    let user_lp_token_account = user.get_or_create_lp_token_account(test).await;
    let user_lp_token_balance_before = test
        .get_token_balance_or_zero(&user_lp_token_account.pubkey)
        .await;

    // Create an AddLiquidity instruction.
    test.builder.add_liquidity(
        &test.state,
        user.keypair.clone(),
        user_lp_token_account.pubkey,
        lamports,
    );
    let result = test.try_execute().await;
    if result.is_err() {
        return result;
    }

    // User SOL account balance decremented.
    let user_sol_balance_after = user.sol_balance(test).await;
    assert_eq!(user_sol_balance_after, user_sol_balance_before - lamports);

    // User's mSOL account was not credited.
    let user_msol_balance_after = test.get_token_balance(&user_msol_account.pubkey).await;
    assert_eq!(user_msol_balance_after, user_msol_balance_before);

    // LP token account is credited with the amount of deposited SOL.
    let user_lp_token_balance_after = test.get_token_balance(&user_lp_token_account.pubkey).await;
    // TODO correctly compute lp_tokens to receive
    assert_eq!(
        user_lp_token_balance_after,
        user_lp_token_balance_before + lamports
    );
    Ok(())
}

pub async fn do_remove_liquidity(
    user: &mut TestUser,
    lp_token_lamports: u64,
    test: &mut IntegrationTest,
) {
    //
    let user_sol_balance_before = user.sol_balance(test).await;

    // TODO: test remove_liquidity when there are msol in the liq_pool
    // The mSOL account is not used.
    // Create a user account for msol if not exists
    let user_msol_account = user.get_or_create_msol_account_instruction(test).await;
    let user_msol_balance_before = test
        .get_token_balance_or_zero(&user_msol_account.pubkey)
        .await;

    // Create a user account for LP tokens if not exists
    let user_lp_token_account = user.get_or_create_lp_token_account(test).await;
    let user_lp_token_balance_before = test
        .get_token_balance_or_zero(&user_lp_token_account.pubkey)
        .await;

    // RemoveLiquidity instruction setup and execution.
    test.builder.remove_liquidity(
        &test.state,
        user_lp_token_account.pubkey,
        user.keypair.clone(),
        user.keypair.pubkey(),
        user_msol_account.pubkey,
        lp_token_lamports,
    );
    test.execute().await;

    // Check post-conditions.
    // TODO: correctly compute SOL to be received
    let user_sol_balance_after = user.sol_balance(test).await;
    assert_eq!(
        user_sol_balance_after,
        user_sol_balance_before + lp_token_lamports
    );

    // TODO: correctly compute mSOL to be received
    let user_msol_balance_after = test.get_token_balance(&user_msol_account.pubkey).await;
    assert_eq!(user_msol_balance_after, user_msol_balance_before);

    // check mSOL removed from user account
    let user_lp_balance_after = test.get_token_balance(&user_lp_token_account.pubkey).await;
    assert_eq!(
        user_lp_balance_after,
        user_lp_token_balance_before - lp_token_lamports
    );
}

#[test(tokio::test)]
async fn test_add_liquidity() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        248, 3, 94, 241, 228, 239, 32, 168, 219, 67, 27, 194, 26, 155, 140, 136, 154, 4, 40, 175,
        132, 80, 60, 31, 135, 250, 230, 19, 172, 106, 254, 120,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;

    let mut alice = test
        .create_test_user("alice", 1001 * LAMPORTS_PER_SOL)
        .await;

    do_add_liquidity(&mut alice, random_amount(1, 1000, &mut rng), &mut test)
        .await
        .unwrap();
    Ok(())
}

#[test(tokio::test)]
async fn test_remove_all_liquidity() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        12, 186, 30, 97, 156, 49, 187, 56, 52, 208, 201, 14, 251, 244, 83, 79, 23, 190, 234, 108,
        198, 232, 147, 111, 207, 188, 128, 153, 82, 236, 69, 88,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;
    let mut user = test
        .create_test_user("alice", 1001 * LAMPORTS_PER_SOL)
        .await;
    let liquidity_amount = random_amount(1, 1000, &mut rng);
    do_add_liquidity(&mut user, liquidity_amount, &mut test)
        .await
        .unwrap();
    do_remove_liquidity(&mut user, liquidity_amount, &mut test).await;
    Ok(())
}
