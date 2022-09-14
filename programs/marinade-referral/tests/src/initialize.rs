use std::sync::Arc;
//use crate::integration_test::Rent;

// use crate::helper::{program_test, IntegrationTest};
//use crate::program_test;
use marinade_finance_offchain_sdk::anchor_lang::prelude::*;
use marinade_finance_offchain_sdk::spl_token;
use marinade_finance_offchain_sdk::{
    instruction_helpers::{initialize::InitializeBuilder, InstructionHelpers},
    marinade_finance::{liq_pool::LiqPool, Fee, State, MAX_REWARD_FEE},
    transaction_builder::TransactionBuilder,
};
use solana_sdk::program_option::COption;

use lazy_static::lazy_static;
use marinade_finance_offchain_sdk::spl_associated_token_account::get_associated_token_address;
//use marinade_reflection::marinade::Marinade;
use rand::{distributions::Uniform, prelude::*};
use rand_chacha::ChaChaRng;
//use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    account::Account,
    instruction::InstructionError,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    //sysvar::clock::{self, Clock},
    transaction::TransactionError,
};
use test_env_log::test;

lazy_static! {
    static ref CREATOR_AUTHORITY: Arc<Keypair> = Arc::new(
        Keypair::from_bytes(&[
            32, 187, 49, 197, 126, 48, 103, 246, 20, 101, 34, 108, 27, 43, 10, 147, 242, 239, 28,
            37, 66, 146, 103, 94, 29, 106, 142, 73, 10, 103, 249, 56, 130, 33, 92, 198, 248, 0, 48,
            210, 221, 172, 150, 104, 107, 227, 44, 217, 3, 61, 74, 58, 179, 76, 35, 104, 39, 67,
            130, 92, 93, 25, 180, 107
        ])
        .unwrap()
    );
}

pub trait InitializeInput {
    fn state(&self) -> Arc<Keypair>;
    fn msol_mint(&self) -> Arc<Keypair>;
    fn admin_authority(&self) -> Arc<Keypair>;
    fn operational_sol_account(&self) -> Pubkey;
    fn validator_manager_authority(&self) -> Arc<Keypair>;
    fn treasury_msol(&self) -> Pubkey;
    fn treasury_msol_authority(&self) -> Pubkey;
    fn build_treasury_msol_account(&self, builder: &mut InitializeBuilder);
    fn lp_mint(&self) -> Arc<Keypair>;
    fn max_stake_count(&self) -> u32;
    fn max_validator_count(&self) -> u32;
    fn reward_fee(&self) -> Fee;
    fn stake_list(&self) -> Pubkey;
    fn build_stake_list(&self, builder: &mut InitializeBuilder, rent: &Rent);
    fn validator_list(&self) -> Pubkey;
    fn build_validator_list(&self, builder: &mut InitializeBuilder, rent: &Rent);
    fn liq_pool_msol_leg(&self) -> Pubkey;
    fn build_liq_pool_msol_leg(&self, builder: &mut InitializeBuilder, rent: &Rent);

    fn build(&self, builder: TransactionBuilder, rent: &Rent) -> TransactionBuilder {
        let mut builder = builder
            .initialize(self.state(), CREATOR_AUTHORITY.clone())
            .unwrap();
        builder.create_msol_mint(self.msol_mint(), &rent);
        builder.set_admin_authority(self.admin_authority().pubkey());
        builder.set_operational_sol_account(self.operational_sol_account());
        builder.use_validator_manager_authority(self.validator_manager_authority().pubkey());
        self.build_treasury_msol_account(&mut builder);
        builder.set_reward_fee(self.reward_fee());
        builder.init_reserve(0, &rent).unwrap();
        builder.create_lp_mint(self.lp_mint(), &rent);
        builder.init_liq_pool_sol_leg(0, &rent).unwrap();

        self.build_stake_list(&mut builder, rent);
        self.build_validator_list(&mut builder, rent);
        self.build_liq_pool_msol_leg(&mut builder, rent);
        builder.build(rent)
    }

    // fn expected_reflection(&self, rent: &Rent, _clock: &Clock) -> Marinade {
    //     let mut builder = marinade_reflection::builder::Builder::default();
    //     let rent_exempt_for_token_acc = rent.minimum_balance(spl_token::state::Account::LEN);
    //     builder.set_msol_mint(self.msol_mint().pubkey());
    //     builder.set_admin_authority(self.admin_authority().pubkey());
    //     builder.set_operational_sol_account(self.operational_sol_account());
    //     builder.set_treasury_msol_account(self.treasury_msol());
    //     builder.set_min_stake(LAMPORTS_PER_SOL);
    //     builder.set_reward_fee(self.reward_fee());
    //     builder.set_validator_manager_authority(self.validator_manager_authority().pubkey());
    //     builder.set_free_validator_slots(self.max_validator_count()); // no used validators and free slots == max validators
    //     builder.set_total_cooling_down(0);
    //     builder.set_cooling_down_stakes(0);
    //     builder.set_free_stake_slots(self.max_stake_count()); // no used stakes and free slots == max stakes
    //     builder.set_lp_mint(self.lp_mint().pubkey());
    //     builder.set_lp_supply(0);
    //     builder.set_actual_liq_pool_sol_amount(rent_exempt_for_token_acc);
    //     builder.set_actual_liq_pool_msol_amount(0);
    //     builder.set_lp_liquidity_target(10000 * LAMPORTS_PER_SOL);
    //     builder.set_lp_max_fee(Fee::from_basis_points(300));
    //     builder.set_lp_min_fee(Fee::from_basis_points(30));
    //     builder.set_lp_treasury_cut(Fee::from_basis_points(2500));
    //     builder.set_available_reserve_balance(0);
    //     builder.set_msol_supply(0);
    //     builder.set_slots_for_stake_delta(24000);
    //     builder.set_last_stake_delta_epoch(Epoch::MAX);
    //     builder.set_min_deposit(1);
    //     builder.set_min_withdraw(1);
    //     builder.build(rent)
    // }
}

// pub async fn check_initialize(
//     input: &impl InitializeInput,
//     banks_client: &mut BanksClient,
//     expected: &Marinade,
// ) -> anyhow::Result<()> {
//     // Check reflection is the same as expected
//     assert_eq!(
//         &Marinade::read_from_test(banks_client, &input.state().pubkey(), vec![]).await?,
//         expected
//     );

//     // Read state again for checking fields not included to reflection
//     let state: State = AccountDeserialize::try_deserialize(
//         &mut banks_client
//             .get_account(input.state().pubkey())
//             .await
//             .unwrap()
//             .unwrap()
//             .data
//             .as_slice(),
//     )?;

//     assert_eq!(*state.stake_system.stake_list_address(), input.stake_list());
//     assert_eq!(
//         *state.validator_system.validator_list_address(),
//         input.validator_list()
//     );
//     assert_eq!(state.liq_pool.msol_leg, input.liq_pool_msol_leg());
//     let stake_list_len = banks_client
//         .get_account(input.stake_list())
//         .await
//         .unwrap()
//         .unwrap()
//         .data
//         .len();
//     let validator_list_len = banks_client
//         .get_account(input.validator_list())
//         .await
//         .unwrap()
//         .unwrap()
//         .data
//         .len();
//     assert_eq!(
//         state
//             .stake_system
//             .stake_list_capacity(stake_list_len)
//             .unwrap(),
//         input.max_stake_count()
//     );
//     assert_eq!(
//         state
//             .validator_system
//             .validator_list_capacity(validator_list_len)
//             .unwrap(),
//         input.max_validator_count()
//     );
//     Ok(())
// }

// Guide for with seed creation process
pub struct InitializeInputWithSeeds {
    pub state: Arc<Keypair>,
    pub msol_mint: Arc<Keypair>,
    pub admin_authority: Arc<Keypair>,
    pub operational_sol_account: Pubkey,
    pub validator_manager_authority: Arc<Keypair>,
    pub treasury_msol_authority: Pubkey,
    pub lp_mint: Arc<Keypair>,

    pub max_stake_count: u32,
    pub max_validator_count: u32,
    pub reward_fee: Fee,
}

impl InitializeInput for InitializeInputWithSeeds {
    fn state(&self) -> Arc<Keypair> {
        self.state.clone()
    }

    fn msol_mint(&self) -> Arc<Keypair> {
        self.msol_mint.clone()
    }

    fn admin_authority(&self) -> Arc<Keypair> {
        self.admin_authority.clone()
    }

    fn operational_sol_account(&self) -> Pubkey {
        self.operational_sol_account
    }

    fn validator_manager_authority(&self) -> Arc<Keypair> {
        self.validator_manager_authority.clone()
    }

    fn treasury_msol(&self) -> Pubkey {
        get_associated_token_address(&self.treasury_msol_authority(), &self.msol_mint().pubkey())
    }

    fn treasury_msol_authority(&self) -> Pubkey {
        self.treasury_msol_authority
    }

    fn build_treasury_msol_account(&self, builder: &mut InitializeBuilder) {
        builder.create_treasury_msol_account(self.treasury_msol_authority());
    }

    fn lp_mint(&self) -> Arc<Keypair> {
        self.lp_mint.clone()
    }

    fn max_stake_count(&self) -> u32 {
        self.max_stake_count
    }

    fn max_validator_count(&self) -> u32 {
        self.max_validator_count
    }

    fn reward_fee(&self) -> Fee {
        self.reward_fee
    }

    fn stake_list(&self) -> Pubkey {
        State::default_stake_list_address(&self.state.pubkey())
    }

    fn build_stake_list(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_stake_list_with_seed(self.max_stake_count, &rent);
    }

    fn validator_list(&self) -> Pubkey {
        State::default_validator_list_address(&self.state.pubkey())
    }

    fn build_validator_list(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_validator_list_with_seed(self.max_validator_count, rent);
    }

    fn liq_pool_msol_leg(&self) -> Pubkey {
        LiqPool::default_msol_leg_address(&self.state.pubkey())
    }

    fn build_liq_pool_msol_leg(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_liq_pool_msol_leg_with_seed(rent);
    }
}

impl InitializeInputWithSeeds {
    pub fn random<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        Self {
            state: Arc::new(Keypair::generate(rng)),
            msol_mint: Arc::new(Keypair::generate(rng)),
            admin_authority: Arc::new(Keypair::generate(rng)),
            operational_sol_account: Pubkey::new_unique(),
            validator_manager_authority: Arc::new(Keypair::generate(rng)),
            treasury_msol_authority: Pubkey::new_unique(),
            lp_mint: Arc::new(Keypair::generate(rng)),
            max_stake_count: Uniform::from(10..=100).sample(rng),
            max_validator_count: Uniform::from(1..=20).sample(rng),
            reward_fee: Fee::from_basis_points(Uniform::from(0..=MAX_REWARD_FEE).sample(rng)),
        }
    }
}

// Guide for without seed creation process
pub struct InitializeInputWithoutSeeds {
    pub with_seeds: InitializeInputWithSeeds,
    pub stake_list: Arc<Keypair>,
    pub validator_list: Arc<Keypair>,
    pub liq_pool_msol_leg: Arc<Keypair>,
}

impl InitializeInput for InitializeInputWithoutSeeds {
    fn state(&self) -> Arc<Keypair> {
        self.with_seeds.state()
    }

    fn msol_mint(&self) -> Arc<Keypair> {
        self.with_seeds.msol_mint()
    }

    fn admin_authority(&self) -> Arc<Keypair> {
        self.with_seeds.admin_authority()
    }

    fn operational_sol_account(&self) -> Pubkey {
        self.with_seeds.operational_sol_account()
    }

    fn validator_manager_authority(&self) -> Arc<Keypair> {
        self.with_seeds.validator_manager_authority()
    }

    fn treasury_msol(&self) -> Pubkey {
        get_associated_token_address(&self.treasury_msol_authority(), &self.msol_mint().pubkey())
    }

    fn treasury_msol_authority(&self) -> Pubkey {
        self.with_seeds.treasury_msol_authority()
    }

    fn build_treasury_msol_account(&self, builder: &mut InitializeBuilder) {
        builder.create_treasury_msol_account(self.treasury_msol_authority());
    }

    fn lp_mint(&self) -> Arc<Keypair> {
        self.with_seeds.lp_mint()
    }

    fn max_stake_count(&self) -> u32 {
        self.with_seeds.max_stake_count()
    }

    fn max_validator_count(&self) -> u32 {
        self.with_seeds.max_validator_count()
    }

    fn reward_fee(&self) -> Fee {
        self.with_seeds.reward_fee()
    }

    fn stake_list(&self) -> Pubkey {
        self.stake_list.pubkey()
    }

    fn build_stake_list(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_stake_list(self.stake_list.clone(), self.max_stake_count(), rent);
    }

    fn validator_list(&self) -> Pubkey {
        self.validator_list.pubkey()
    }

    fn build_validator_list(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_validator_list(
            self.validator_list.clone(),
            self.max_validator_count(),
            rent,
        );
    }

    fn liq_pool_msol_leg(&self) -> Pubkey {
        self.liq_pool_msol_leg.pubkey()
    }

    fn build_liq_pool_msol_leg(&self, builder: &mut InitializeBuilder, rent: &Rent) {
        builder.create_liq_pool_msol_leg(self.liq_pool_msol_leg.clone(), rent);
    }
}

impl InitializeInputWithoutSeeds {
    pub fn random<R: CryptoRng + RngCore>(rng: &mut R) -> Self {
        Self {
            with_seeds: InitializeInputWithSeeds::random(rng),
            stake_list: Arc::new(Keypair::generate(rng)),
            validator_list: Arc::new(Keypair::generate(rng)),
            liq_pool_msol_leg: Arc::new(Keypair::generate(rng)),
        }
    }
}

/*#[test(tokio::test)]
async fn test_initialization_without_seeds() -> anyhow::Result<()> {
    let (mut banks_client, payer, recent_blockhash) = crate::program_test().start().await;
    let fee_payer = Arc::new(payer);
    let rent = banks_client.get_rent().await?;
    // let clock: Clock =
    //     bincode::deserialize(&banks_client.get_account(clock::ID).await?.unwrap().data)?;

    use rand_chacha::rand_core::SeedableRng;
    let mut rng = ChaChaRng::from_seed([
        102, 46, 250, 122, 194, 179, 201, 43, 230, 4, 42, 246, 158, 90, 248, 237, 8, 61, 81, 114,
        227, 137, 83, 10, 40, 93, 233, 9, 35, 24, 77, 213,
    ]);
    let input = InitializeInputWithoutSeeds::random(&mut rng);
    //let expected = input.expected_reflection(&rent, &clock);

    let builder = TransactionBuilder::unlimited(fee_payer);

    let transaction = input
        .build(builder, &rent)
        .build_one_combined()
        .unwrap()
        .into_signed(recent_blockhash)?;

    banks_client.process_transaction(transaction).await?;

    // let state: State = AccountDeserialize::try_deserialize(
    //     &mut banks_client
    //         .get_account(input.with_seeds.state.pubkey())
    //         .await?
    //         .unwrap()
    //         .data
    //         .as_slice(),
    // )?;
    //check_initialize(&input, &mut banks_client, &expected).await?;
    Ok(())
}

#[test(tokio::test)]
async fn test_initialization_with_seeds() -> anyhow::Result<()> {
    let (mut banks_client, payer, recent_blockhash) = crate::program_test().start().await;
    let fee_payer = Arc::new(payer);
    let rent = banks_client.get_rent().await?;
    // let clock: Clock =
    //     bincode::deserialize(&banks_client.get_account(clock::ID).await?.unwrap().data)?;

    let mut rng = ChaChaRng::from_seed([
        165, 207, 247, 199, 219, 129, 37, 113, 150, 217, 64, 249, 26, 198, 236, 23, 14, 109, 38,
        112, 152, 203, 30, 106, 214, 229, 34, 192, 20, 141, 116, 234,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);
    //let expected = input.expected_reflection(&rent, &clock);

    let builder = TransactionBuilder::unlimited(fee_payer);
    let transaction = input
        .build(builder, &rent)
        .build_one_combined()
        .unwrap()
        .into_signed(recent_blockhash)?;

    banks_client.process_transaction(transaction).await.unwrap();

    // let state: State = AccountDeserialize::try_deserialize(
    //     &mut banks_client
    //         .get_account(input.state.pubkey())
    //         .await?
    //         .unwrap()
    //         .data
    //         .as_slice(),
    // )?;
    //check_initialize(&input, &mut banks_client, &expected).await?;
    Ok(())
}
*/

/*
#[test(tokio::test)]
async fn test_empty_reserve() -> anyhow::Result<()> {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;
    let fee_payer = Arc::new(payer);
    let rent = banks_client.get_rent().await?;

    let mut rng = ChaChaRng::from_seed([
        159, 223, 172, 160, 99, 98, 67, 97, 50, 252, 75, 173, 149, 169, 58, 142, 110, 68, 63, 166,
        118, 32, 251, 6, 195, 18, 123, 22, 164, 220, 39, 70,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);

    let builder = TransactionBuilder::unlimited(fee_payer);
    let mut builder = builder
        .initialize(input.state.clone(), CREATOR_AUTHORITY.clone())
        .unwrap();
    builder.create_msol_mint(input.msol_mint.clone(), &rent);
    builder.set_admin_authority(input.admin_authority.pubkey());
    builder.set_operational_sol_account(input.operational_sol_account);
    builder.use_validator_manager_authority(input.validator_manager_authority.pubkey());
    builder.create_treasury_msol_account(input.treasury_msol_authority());
    builder.set_reward_fee(input.reward_fee);
    builder.assume_reserve_initialized(); // <- error
    builder.create_lp_mint(input.lp_mint.clone(), &rent);
    builder.init_liq_pool_sol_leg(0, &rent)?;
    builder.create_stake_list_with_seed(input.max_stake_count, &rent);
    builder.create_validator_list_with_seed(input.max_validator_count, &rent);
    builder.create_liq_pool_msol_leg_with_seed(&rent);

    let transaction = builder
        .build(&rent)
        .build_one_combined()
        .unwrap()
        .into_signed(recent_blockhash)?;

    assert_eq!(
        banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(11, InstructionError::InvalidArgument)
    );
    Ok(())
}
*/

/*
#[test(tokio::test)]
async fn test_fake_mint() -> anyhow::Result<()> {
    let mut test = program_test();
    let mut rng = ChaChaRng::from_seed([
        48, 200, 13, 148, 221, 72, 28, 60, 241, 255, 51, 45, 19, 65, 135, 178, 209, 153, 103, 95,
        240, 47, 73, 17, 175, 231, 216, 145, 187, 222, 130, 183,
    ]);
    let input = InitializeInputWithSeeds::random(&mut rng);

    let mut fake_mint_account =
        Account::new(10000000, spl_token::state::Mint::LEN, &Pubkey::new_unique());
    let mint_state = spl_token::state::Mint {
        mint_authority: COption::Some(State::find_msol_mint_authority(&input.state.pubkey()).0),
        supply: 0,
        decimals: 9,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    mint_state.pack_into_slice(&mut fake_mint_account.data);
    test.add_account(input.msol_mint.pubkey(), fake_mint_account);

    let (mut banks_client, payer, recent_blockhash) = test.start().await;
    let rent = banks_client.get_rent().await.unwrap();
    let fee_payer = Arc::new(payer);

    let builder = TransactionBuilder::unlimited(fee_payer);
    let mut builder = builder
        .initialize(input.state.clone(), CREATOR_AUTHORITY.clone())
        .unwrap();
    builder.use_msol_mint_pubkey(input.msol_mint.pubkey());
    builder.set_admin_authority(input.admin_authority.pubkey());
    builder.set_operational_sol_account(input.operational_sol_account);
    builder.use_validator_manager_authority(input.validator_manager_authority.pubkey());
    builder.create_treasury_msol_account(input.treasury_msol_authority());
    builder.set_reward_fee(input.reward_fee);
    builder.init_reserve(0, &rent)?;
    builder.create_lp_mint(input.lp_mint.clone(), &rent);
    builder.init_liq_pool_sol_leg(0, &rent)?;
    builder.create_stake_list_with_seed(input.max_stake_count, &rent);
    builder.create_validator_list_with_seed(input.max_validator_count, &rent);
    builder.create_liq_pool_msol_leg_with_seed(&rent);

    let transaction = builder
        .build(&rent)
        .build_one_combined()
        .unwrap()
        .into_signed(recent_blockhash)?;
    assert_eq!(
        banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(10, InstructionError::InvalidArgument)
    );
    Ok(())
}
*/
