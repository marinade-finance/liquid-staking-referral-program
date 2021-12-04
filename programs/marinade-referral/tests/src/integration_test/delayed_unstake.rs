#![allow(unused_imports)]
use crate::{initialize::InitializeInputWithSeeds, integration_test::IntegrationTest};

use marinade_finance_offchain_sdk::anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use marinade_finance_offchain_sdk::marinade_finance;
use marinade_finance_offchain_sdk::spl_associated_token_account::get_associated_token_address;
use marinade_finance_offchain_sdk::{
    instruction_helpers::InstructionHelpers,
    marinade_finance::{ticket_account::TicketAccountData, State},
};
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
use solana_sdk::{
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::sync::Arc;
use test_env_log::test;

use log::*;

pub struct DelayedUnstakeParams {
    ///user sol main account
    user_sol: Arc<Keypair>,
    user_sol_initial_balance: u64,
    user_msol_address: Pubkey,
    ticket_account: Arc<Keypair>,
    ticket_account_rent_exempt_lamports: u64,
    initial_stake_amount: u64,
    msol_burn_amount: u64,
    expected_lamports: u64,
}

impl DelayedUnstakeParams {
    pub fn new(state: &State) -> Self {
        let user_sol_keypair = Keypair::new();
        let user_pubkey = user_sol_keypair.pubkey();
        Self {
            user_sol: Arc::new(user_sol_keypair),
            user_sol_initial_balance: 100 * LAMPORTS_PER_SOL,
            user_msol_address: get_associated_token_address(&user_pubkey, &state.msol_mint),
            initial_stake_amount: 10 * LAMPORTS_PER_SOL,
            msol_burn_amount: 5 * LAMPORTS_PER_SOL,
            ticket_account: Arc::new(Keypair::new()),
            ticket_account_rent_exempt_lamports: 0,
            expected_lamports: 0,
        }
    }
}

pub async fn do_order_unstake(params: &mut DelayedUnstakeParams, test: &mut IntegrationTest) {
    //
    println!("--");
    println!("--");
    println!("START user_sol_acc:{}", params.user_sol.pubkey());
    // put some SOL into users' account.
    test.builder
        .transfer_lamports(
            test.fee_payer_signer(),
            &params.user_sol.pubkey(),
            params.user_sol_initial_balance,
            "fee payer",
            "user SOL",
        )
        .unwrap();

    // Create a user account for mSOL.
    let user_msol = test
        .builder
        .create_associated_token_account(
            &params.user_sol.pubkey(),
            &test.state.msol_mint,
            "user mSOL",
        )
        .unwrap();
    assert_eq!(user_msol, params.user_msol_address);

    // // Create a user account for LP tokens.
    // let user_lp = test
    //     .builder
    //     .create_associated_token_account(
    //         &params.user_sol.pubkey(),
    //         &test.state.liq_pool.lp_mint,
    //         "user lp",
    //     )
    //     .unwrap();
    // assert_eq!(user_lp, params.user_lp(&test.state));

    test.execute().await;
    println!("\n\n---- ACCOUNTS CREATED");

    // // add an AddLiquidity instruction.
    // test.builder.add_liquidity(
    //     &test.state,
    //     params.user_sol.clone(),
    //     params.user_lp(&test.state),
    //     params.initial_stake_amount,
    // );
    // test.execute().await;
    // println!("------- LIQ ADDED");

    //stake instruction
    test.builder.deposit(
        &test.state,
        params.user_sol.clone(),
        params.user_msol_address,
        params.initial_stake_amount,
    );
    //execute until here
    //this will be initial state
    //for testing unstake_order
    test.execute().await;
    println!("\n\n------------ STAKED");

    let user_msol_balance_after = test.get_token_balance(&params.user_msol_address).await;
    println!(
        "-------- user_msol_balance_after {}",
        user_msol_balance_after
    );
    debug_assert_eq!(user_msol_balance_after, params.initial_stake_amount);

    // Create a empty ticket account (transfer rent-exempt lamports)
    const TICKET_ACCOUNT_SPACE: usize = 8 + std::mem::size_of::<TicketAccountData>();
    params.ticket_account_rent_exempt_lamports = test.rent.minimum_balance(TICKET_ACCOUNT_SPACE);
    let _ticket_account = test
        .builder
        .create_account(
            params.ticket_account.clone(),
            TICKET_ACCOUNT_SPACE,
            &marinade_finance::ID,
            &test.rent,
            "ticket-account",
        )
        .unwrap();

    params.expected_lamports = test
        .state
        .calc_lamports_from_msol_amount(params.msol_burn_amount)
        .unwrap();

    // Create a OrderUnstake instruction.
    test.builder.order_unstake(
        &test.state,
        params.user_msol_address,
        params.user_sol.clone(), //user_msol owner & signer
        params.msol_burn_amount,
        params.ticket_account.pubkey(),
        // params.user_sol.pubkey(), //ticket beneficiary
    );
    test.execute().await;

    // User SOL account balance decremented (rent-exempt & fees)
    let user_sol_balance_after = test
        .context
        .banks_client
        .get_balance(params.user_sol.pubkey())
        .await
        .unwrap();
    println!("user_sol_balance_after {}", user_sol_balance_after);
    debug_assert_eq!(
        user_sol_balance_after,
        params.user_sol_initial_balance - params.initial_stake_amount
    );

    // read Ticket account data
    let ticket: TicketAccountData = test.get_account_data(&params.ticket_account.pubkey()).await;

    debug_assert_eq!(ticket.beneficiary, params.user_sol.pubkey());
    debug_assert_eq!(ticket.lamports_amount, params.expected_lamports);
    debug_assert_eq!(ticket.created_epoch, test.clock.epoch);
}

//-----------------------------
//-----------------------------
//-----------------------------
pub async fn do_claim(params: &mut DelayedUnstakeParams, test: &mut IntegrationTest) {
    //
    // read Ticket account data
    let ticket: TicketAccountData = test.get_account_data(&params.ticket_account.pubkey()).await;

    debug_assert_eq!(ticket.beneficiary, params.user_sol.pubkey());
    debug_assert_eq!(ticket.lamports_amount, params.expected_lamports);
    debug_assert!(ticket.created_epoch <= test.clock.epoch);

    // epoch boundaries
    // for i in 1..=100 {
    //     println!("i{}",i);
    //     test.move_to_slot(i*5).await;
    // }
    // return;

    params.expected_lamports = test
        .state
        .calc_lamports_from_msol_amount(params.msol_burn_amount)
        .unwrap();

    let user_sol_address = params.user_sol.pubkey().clone();
    let pre_balance = test.get_sol_balance(&user_sol_address).await;

    // Create a Claim instruction.
    test.builder.claim(
        &test.state,
        params.ticket_account.pubkey(),
        params.user_sol.pubkey(), //ticket beneficiary
    );
    // should fail with NOT-DUE
    const ERR_CODE_TICKET_NOT_DUE: u32 = 0x1103;
    println!("epoch:{}, slot:{}", test.clock.epoch, test.clock.slot);
    match test.try_execute().await {
        Ok(()) => debug_assert!(false, "expected err got Ok"),
        Err(ERR_CODE_TICKET_NOT_DUE) => {
            println!("(expected tx failure 0x{:x})", ERR_CODE_TICKET_NOT_DUE)
        }
        Err(x) => debug_assert!(false, "expected err(2) got 0x{:x}", x),
    }

    //how many slots per epoch?
    //let warp_to = test.context.genesis_config().epoch_schedule.slots_per_epoch;
    let slots_per_epoch = 100;
    let mut clock = test.move_to_slot(2 * slots_per_epoch).await;
    debug_assert_eq!(clock.epoch, 2);

    // another Claim instruction.
    test.builder.claim(
        &test.state,
        params.ticket_account.pubkey(),
        params.user_sol.pubkey(), //ticket beneficiary
    );
    // should fail with NOT_READY_WAIT
    const ERR_CODE_TICKET_NOT_READY_WAIT: u32 = 0x1104;
    match test.try_execute().await {
        Ok(()) => debug_assert!(false, "expected err got Ok"),
        Err(ERR_CODE_TICKET_NOT_READY_WAIT) => println!(
            "(expected tx failure 0x{:x})",
            ERR_CODE_TICKET_NOT_READY_WAIT
        ),
        Err(x) => debug_assert!(
            false,
            "expected err(ERR_CODE_TICKET_NOT_READY_WAIT) got 0x{:x}",
            x
        ),
    }

    //move to epoch 3
    clock = test.move_to_slot(470).await;
    debug_assert_eq!(clock.epoch, 3);

    let other_account = Keypair::new();
    // another Claim instruction.
    test.builder.claim(
        &test.state,
        params.ticket_account.pubkey(),
        other_account.pubkey(), //ticket beneficiary (wrong)
    );
    // should fail with WRONG_BENEFICIARY
    const ERR_CODE_TICKET_WRONG_BENEFICIARY: u32 = 0x1105;
    match test.try_execute().await {
        Ok(()) => debug_assert!(false, "expected err got Ok"),
        Err(ERR_CODE_TICKET_WRONG_BENEFICIARY) => println!(
            "(expected tx failure 0x{:x})",
            ERR_CODE_TICKET_WRONG_BENEFICIARY
        ),
        Err(x) => debug_assert!(
            false,
            "expected err(ERR_CODE_TICKET_WRONG_BENEFICIARY) got 0x{:x}",
            x
        ),
    }

    // another Claim instruction.
    test.builder.claim(
        &test.state,
        params.ticket_account.pubkey(),
        params.user_sol.pubkey(), //ticket beneficiary
    );
    // should NOT fail
    test.execute().await;

    let post_balance = test.get_sol_balance(&user_sol_address).await;
    println!(
        "pre-bal:{}, post-bal:{}, expected lamports:{}, ticket_account_rent_exempt_lamports:{}",
        pre_balance,
        post_balance,
        params.expected_lamports,
        params.ticket_account_rent_exempt_lamports
    );
    //user also gets params.ticket_account_rent_exempt_lamports
    debug_assert_eq!(
        post_balance,
        pre_balance + params.expected_lamports + params.ticket_account_rent_exempt_lamports
    );
}

#[test(tokio::test)]
async fn test_order_unstake() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        16, 92, 208, 54, 210, 197, 121, 104, 206, 85, 87, 224, 82, 254, 190, 21, 215, 100, 27, 49,
        242, 245, 119, 105, 132, 147, 177, 94, 108, 204, 101, 124,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;
    let mut params = DelayedUnstakeParams::new(&test.state);
    do_order_unstake(&mut params, &mut test).await;
    Ok(())
}

#[test(tokio::test)]
async fn test_claim_ticket() -> anyhow::Result<()> {
    let mut rng = ChaChaRng::from_seed([
        146, 11, 44, 41, 184, 232, 84, 155, 176, 163, 71, 218, 115, 107, 89, 127, 156, 1, 33, 128,
        78, 186, 56, 59, 151, 2, 15, 144, 136, 25, 66, 57,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    let mut test = IntegrationTest::start(&input).await?;
    let mut params = DelayedUnstakeParams::new(&test.state);
    do_order_unstake(&mut params, &mut test).await;
    do_claim(&mut params, &mut test).await;
    Ok(())
}
