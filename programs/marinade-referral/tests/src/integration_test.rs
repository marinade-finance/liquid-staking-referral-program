#![allow(dead_code)]
//use assert_json_diff::assert_json_eq;
use marinade_finance_offchain_sdk::anchor_lang::InstructionData;
use marinade_finance_offchain_sdk::marinade_finance;
use marinade_finance_offchain_sdk::spl_token::solana_program;
// use marinade_reflection::accounts_builder::AccountsBuilder;
// use marinade_reflection::marinade::Marinade;
use rand::{distributions::Uniform, prelude::Distribution, RngCore};
use std::collections::{HashMap, HashSet};
use std::io::{self};
use std::sync::Arc;

use futures::{Future, FutureExt};
use marinade_finance_offchain_sdk::anchor_lang::solana_program::{
    native_token::{lamports_to_sol, LAMPORTS_PER_SOL},
    program_pack::Pack,
    pubkey::Pubkey,
    stake,
    stake::state::{Authorized, Lockup, StakeState},
    system_instruction, system_program, sysvar,
};
use marinade_finance_offchain_sdk::instruction_helpers::InstructionHelpers;
use solana_sdk::{account::from_account, instruction::Instruction, transaction::Transaction};

use marinade_finance_offchain_sdk::anchor_lang::prelude::*;
use marinade_finance_offchain_sdk::{
    marinade_finance::located::Located, marinade_finance::State,
    transaction_builder::TransactionBuilder, WithKey,
};

use solana_program_test::{processor, BanksClient, ProgramTest, ProgramTestContext};

use anyhow::anyhow;
use marinade_finance_offchain_sdk::spl_associated_token_account::get_associated_token_address;
use marinade_finance_offchain_sdk::spl_token::state::Account as TokenAccount;
use solana_sdk::{
    fee_calculator::FeeCalculator,
    hash::Hash,
    rent::Rent,
    signature::{Keypair, Signer},
    stake::state::Stake,
};
use solana_vote_program::{
    vote_instruction,
    vote_state::{VoteInit, VoteState},
};

use crate::initialize::InitializeInput;

pub mod test_add_remove_liquidity;
pub mod test_delayed_unstake;
pub mod test_deposit_sol_liquid_unstake;
pub mod test_deposit_stake_account;

pub struct StakeInfo {
    pub index: u32,
    pub state: Stake,
    pub last_update_delegated_lamports: u64,
    pub last_update_epoch: u64,
    pub actual_balance: u64,
}

/// Common parameters of an integration test.
pub struct IntegrationTest {
    pub context: ProgramTestContext,
    pub rent: Rent,
    pub clock: Clock,
    pub builder: TransactionBuilder,
    pub state: WithKey<State>, // marinade_finance state
    // Individual stakes are not present in reflection but in some tests we need to check it
    pub stakes: HashMap<Pubkey, StakeInfo>,
    //pub reflection: Marinade,
    pub admin_authority: Arc<Keypair>,
    pub validator_manager_authority: Arc<Keypair>,
    pub claim_ticket_accounts: HashSet<Pubkey>,
    //
    test_validators: Vec<TestValidator>,
}

#[derive(Debug)]
pub struct TestValidator {
    pub keypair: Arc<Keypair>,
    pub vote_keypair: Arc<Keypair>,
    pub name: String,
}
impl TestValidator {
    pub fn new(name: &str) -> Self {
        Self {
            keypair: Arc::new(Keypair::new()),
            vote_keypair: Arc::new(Keypair::new()),
            name: name.into(),
        }
    }
}

#[derive(Debug)]
pub struct TestTokenAccount {
    pub symbol: String,
    pub pubkey: Pubkey,
    pub user_name: String,
}

pub struct TestUser {
    pub name: String,
    pub keypair: Arc<Keypair>,
}

impl TestUser {
    pub async fn sol_balance(&self, test: &mut IntegrationTest) -> u64 {
        test.get_sol_balance(&self.keypair.pubkey()).await
    }

    pub fn msol_account_pubkey(&mut self, test: &mut IntegrationTest) -> Pubkey {
        const SYMBOL: &str = "mSOL";
        let mint = test.mint_from_symbol(SYMBOL);
        get_associated_token_address(&self.keypair.pubkey(), mint)
    }

    pub async fn get_or_create_msol_account_instruction(
        &self,
        test: &mut IntegrationTest,
    ) -> TestTokenAccount {
        const SYMBOL: &str = "mSOL";
        return TestTokenAccount {
            symbol: String::from(SYMBOL),
            pubkey: test
                .get_or_create_associated_token_account(&self, SYMBOL)
                .await,
            user_name: self.name.clone(),
        };
    }

    pub fn lp_token_account_pubkey(&mut self, test: &mut IntegrationTest) -> Pubkey {
        const SYMBOL: &str = "mSOL-SOL-LP";
        let mint = test.mint_from_symbol(SYMBOL);
        get_associated_token_address(&self.keypair.pubkey(), mint)
    }

    pub async fn get_or_create_lp_token_account(
        &self,
        test: &mut IntegrationTest,
    ) -> TestTokenAccount {
        const SYMBOL: &str = "mSOL-SOL-LP";
        return TestTokenAccount {
            symbol: String::from(SYMBOL),
            pubkey: test
                .get_or_create_associated_token_account(&self, SYMBOL)
                .await,
            user_name: self.name.clone(),
        };
    }
}

impl IntegrationTest {
    /// Starts an integration test and initializes the common parameters.
    pub async fn start(input: &impl InitializeInput) -> anyhow::Result<Self> {
        let mut main_test_program = ProgramTest::new(
            "marinade_finance",
            marinade_finance::ID,
            processor!(marinade_finance::test_entry),
        );

        main_test_program.add_program(
            "marinade_referral",
            marinade_referral::marinade_referral::ID,
            processor!(marinade_referral::marinade_referral::test_entry),
            //None, //processor!(marinade_referral::test_entry),
        );
        // let marinade_referral =
        //     ProgramTest::new(
        //         "marinade_referral",
        //         marinade_referral::marinade_referral::ID,
        //         None //processor!(marinade_referral::test_entry),
        //     );

        let mut context = main_test_program.start_with_context().await;
        //let (mut banks_client, payer, recent_blockhash) = test.start().await;

        let rent = context.banks_client.get_rent().await?;
        let clock = get_clock(&mut context.banks_client).await?;
        //let expected = input.expected_reflection(&rent, &clock);
        //clone main account keypair
        let fee_payer = Arc::new(Keypair::from_bytes(&context.payer.to_bytes())?);

        // Set up the required instruction sequence for MARINADE-FINANCE-LIQUID-STAKE-PROGRAM initialization.
        let builder = TransactionBuilder::unlimited(fee_payer);
        let mut builder = input.build(builder, &rent);
        let transaction = builder
            .build_one_combined()
            .unwrap()
            .into_signed(context.last_blockhash)
            .unwrap();

        // execute MARINADE-FINANCE-LIQUID-STAKE-PROGRAM initialization
        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        // read MARINADE-FINANCE-LIQUID-STAKE-PROGRAM state
        let state: State = AccountDeserialize::try_deserialize(
            &mut context
                .banks_client
                .get_account(input.state().pubkey())
                .await?
                .unwrap()
                .data
                .as_slice(),
        )
        .unwrap();

        let state = WithKey::new(state, input.state().pubkey());

        let stakes = Self::read_stakes(state.as_ref(), &mut context.banks_client).await?;

        Ok(IntegrationTest {
            context,
            rent,
            clock,
            builder,
            state,
            stakes,
            //reflection: expected,
            admin_authority: input.admin_authority(),
            validator_manager_authority: input.validator_manager_authority(),
            claim_ticket_accounts: HashSet::new(),
            test_validators: vec![],
        })

        // referral program, initialize global state
        /*
        //pub fn initialize_global_state()
            // referral global state PDA & bump
            let (global_state_pda, bump) = Pubkey::find_program_address(
                &[GLOBAL_STATE_SEED],
                &marinade_referral::marinade_referral::ID
                );

            // initialize the global state
            {
                let accounts = marinade_referral::accounts::Initialize {
                    admin_account: Pubkey::from_str("AMMK9YLj8PRRG4K9DUsTNPZAZXeVbHiQJxakuVuvSKrn")
                        .unwrap(),
                    global_state: global_state_pda,
                    system_program: system_program::ID,
                };
            }
        }
        pub fn initialize_referral(partner_main_account:Pubkey){
                // referral state PDA & bump
                let (referralStatePda, referralStateBump) = Pubkey::find_program_address(
                    &[
                        &partner_main_account.to_bytes()[..32],
                        &REFERRAL_STATE_SEED,
                    ],
                    &marinade_referral::marinade_referral::ID
                    );
            }
        */
    }

    // pub async fn start_synthetic(
    //     reflection_account_builder: &AccountsBuilder<'_>,
    //     additional_accounts: HashMap<Pubkey, Account>,
    //     rng: &mut (impl RngCore + CryptoRng),
    // ) -> anyhow::Result<Self> {
    //     let admin_authority = Arc::new(Keypair::generate(rng));
    //     let validator_manager_authority = Arc::new(Keypair::generate(rng));
    //     let mut test = program_test();
    //     let rent = Rent::default(); // must be ok
    //     let accounts = reflection_account_builder.build(&rent)?;
    //     let mut initial_accounts: HashMap<Pubkey, Account> = accounts.storage.clone();
    //     for (key, account) in additional_accounts {
    //         if initial_accounts.insert(key, account).is_some() {
    //             bail!("Additional account {} duplicated", key);
    //         }
    //     }

    //     for (key, account) in initial_accounts {
    //         test.add_account(key, account);
    //     }

    //     let mut context = test.start_with_context().await;

    //     let actual_rent = context.banks_client.get_rent().await?;
    //     assert_eq!(actual_rent, rent);
    //     let clock = get_clock(&mut context.banks_client).await?;
    //     // clone main account keypair
    //     let fee_payer = Arc::new(Keypair::from_bytes(&context.payer.to_bytes()).unwrap());

    //     // let start_reflection = Marinade::read_from_test(
    //     //     &mut context.banks_client,
    //     //     &reflection_account_builder.instance,
    //     //     reflection_account_builder
    //     //         .marinade
    //     //         .claim_ticket_keys::<Vec<Pubkey>>(),
    //     // )
    //     // .await?;
    //     // assert_json_eq!(&start_reflection, reflection_account_builder.marinade);

    //     let builder = TransactionBuilder::unlimited(fee_payer);

    //     let state: State = AccountDeserialize::try_deserialize(
    //         &mut context
    //             .banks_client
    //             .get_account(reflection_account_builder.instance)
    //             .await?
    //             .unwrap()
    //             .data
    //             .as_slice(),
    //     )?;

    //     let state = WithKey::new(state, reflection_account_builder.instance);
    //     let stakes = Self::read_stakes(state.as_ref(), &mut context.banks_client).await?;

    //     let mut result = IntegrationTest {
    //         context,
    //         rent,
    //         clock,
    //         builder,
    //         state,
    //         stakes,
    //         reflection: reflection_account_builder.marinade.clone(),
    //         admin_authority,
    //         validator_manager_authority,
    //         claim_ticket_accounts: reflection_account_builder.marinade.claim_ticket_keys(),
    //         test_validators: vec![],
    //     };

    //     let epoch_schedule = result.context.genesis_config().epoch_schedule;

    //     result
    //         .move_to_slot(epoch_schedule.get_first_slot_in_epoch(accounts.target_epoch))
    //         .await;

    //     Ok(result)
    // }

    async fn read_stakes(
        state: &State,
        banks_client: &mut BanksClient,
    ) -> anyhow::Result<HashMap<Pubkey, StakeInfo>> {
        let mut stakes = HashMap::new();
        let stake_list = banks_client
            .get_account(*state.stake_system.stake_list_address())
            .await?
            .ok_or_else(|| {
                anyhow!(
                    "Marinade validator list {} not found",
                    state.stake_system.stake_list_address()
                )
            })?
            .data;
        for i in 0..state.stake_system.stake_count() {
            let stake_record = state.stake_system.get(&stake_list, i)?;

            let stake_account = banks_client
                .get_account(stake_record.stake_account)
                .await?
                .ok_or_else(|| {
                    anyhow!("Marinade stake {} not found", stake_record.stake_account)
                })?;

            let stake_state =
                bincode::deserialize::<StakeState>(&stake_account.data).map_err(|err| {
                    anyhow!(
                        "Error reading stake {}: {}",
                        stake_record.stake_account,
                        err
                    )
                })?;

            // let stake_delegation = stake_state.delegation().ok_or_else(|| {
            //     anyhow!(
            //         "Undelegated stake {} under control",
            //         stake_record.stake_account
            //     )
            // })?;

            stakes.insert(
                stake_record.stake_account,
                StakeInfo {
                    index: i,
                    state: stake_state.stake().unwrap(),
                    last_update_delegated_lamports: stake_record.last_update_delegated_lamports,
                    last_update_epoch: stake_record.last_update_epoch,
                    actual_balance: stake_account.lamports,
                },
            );
        }

        Ok(stakes)
    }

    /// Computes the recent blockhash.
    pub async fn recent_blockhash(&mut self) -> Hash {
        self.context
            .banks_client
            .get_recent_blockhash()
            .await
            .unwrap()
    }

    pub async fn update_state(&mut self) -> anyhow::Result<()> {
        let key = self.state.key();
        self.state.replace(
            AccountDeserialize::try_deserialize(
                &mut self
                    .context
                    .banks_client
                    .get_account(key)
                    .await
                    .unwrap()
                    .unwrap()
                    .data
                    .as_slice(),
            )
            .unwrap(),
        );
        // self.reflection = Marinade::read_from_test(
        //     &mut self.context.banks_client,
        //     &key,
        //     self.claim_ticket_accounts.clone(),
        // )
        // .await?;
        self.stakes =
            Self::read_stakes(self.state.as_ref(), &mut self.context.banks_client).await?;

        Ok(())
    }

    pub fn fee_payer(&self) -> Pubkey {
        self.builder.fee_payer()
    }

    pub fn fee_payer_signer(&self) -> Arc<dyn Signer> {
        self.builder.fee_payer_signer()
    }

    pub async fn fee_calculator(&mut self) -> FeeCalculator {
        self.context.banks_client.get_fees().await.unwrap().0
    }

    pub async fn execute_txn(
        &mut self,
        mut transaction: Transaction,
        signers: Vec<Arc<dyn Signer>>,
    ) {
        let recent_blockhash = self.recent_blockhash().await;
        println!("signers len()={}", &signers.len());
        transaction
            .try_sign(
                &signers.iter().map(|arc| arc.as_ref()).collect::<Vec<_>>(),
                recent_blockhash,
            )
            .unwrap();

        self.context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        self.update_state().await.unwrap();
    }

    pub async fn execute_instruction(
        &mut self,
        instruction: Instruction,
        signers: Vec<Arc<dyn Signer>>,
    ) {
        let tx = Transaction::new_with_payer(&[instruction], Some(&signers[0].pubkey()));
        // marinade-referral execution
        self.execute_txn(tx, signers).await;
    }

    pub async fn execute(&mut self) {
        let transaction = self.builder.build_one_combined();
        let transaction = if let Some(transaction) = transaction {
            transaction
        } else {
            return; // Nothing to do
        };

        println!("--- Run transaction with instructions:");
        for (index, description) in transaction.instruction_descriptions.iter().enumerate() {
            println!("Instruction #{}: {}", index, description);
        }

        let transaction = transaction
            .into_signed(self.recent_blockhash().await)
            .unwrap();

        self.context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();
        self.update_state().await.unwrap();
    }

    pub async fn try_execute(&mut self) -> Result<(), u32> {
        let transaction = self.builder.build_one_combined();
        let transaction = if let Some(transaction) = transaction {
            transaction
        } else {
            return Ok(()); // Nothing to do
        };

        println!("--- try Run transaction with instructions:");
        for (index, description) in transaction.instruction_descriptions.iter().enumerate() {
            println!("Instruction #{}: {}", index, description);
        }

        // Warning: do not use self.context.last_blockhash
        // because is not updated, and if you re-try the same transaction twice (and use self.context.last_blockhash which is the old blockhash)
        // .banks_client.process_transaction/Solana core WILL NOT execute the txn,
        // (maybe it is considering this a re-send of the same tx?)
        // and will just RETURN THE CACHED RESULT from the first tx
        // so we use self.context.banks_client.get_recent_blockhash() here
        let blockhash = self
            .context
            .banks_client
            .get_recent_blockhash()
            .await
            .unwrap();
        //println!("context.last_blockhash {:?} banks_client.get_recent_blockhash() {:?}",self.context.last_blockhash, blockhash);
        //let transaction = transaction.into_signed(self.context.last_blockhash); <-- DO NOT USE self.context.last_blockhash
        let transaction = transaction.into_signed(blockhash);
        if transaction.is_err() {
            return Err(1);
        }

        let tx_result = self
            .context
            .banks_client
            .process_transaction(transaction.unwrap())
            .await;

        println!("try execute result {:x?}", tx_result);
        if let Err(transport_error) = tx_result {
            if let solana_sdk::transport::TransportError::TransactionError(transaction_error) =
                transport_error
            {
                if let solana_sdk::transaction::TransactionError::InstructionError(
                    _index,
                    instruction_error,
                ) = transaction_error
                {
                    if let solana_program::instruction::InstructionError::Custom(code) =
                        instruction_error
                    {
                        return Err(code);
                    }
                }
            }
            return Err(2);
        }

        self.update_state().await.unwrap();
        Ok(())
    }

    // returns Result
    pub async fn try_execute_txn(
        &mut self,
        mut transaction: Transaction,
        signers: Vec<Arc<dyn Signer>>,
    ) -> Result<(), u32> {
        let recent_blockhash = self.recent_blockhash().await;
        println!("signers len()={}", &signers.len());
        transaction
            .try_sign(
                &signers.iter().map(|arc| arc.as_ref()).collect::<Vec<_>>(),
                recent_blockhash,
            )
            .unwrap();

        let tx_result = self
            .context
            .banks_client
            .process_transaction(transaction)
            .await;

        println!("try execute result {:x?}", tx_result);
        if let Err(transport_error) = tx_result {
            if let solana_sdk::transport::TransportError::TransactionError(transaction_error) =
                transport_error
            {
                if let solana_sdk::transaction::TransactionError::InstructionError(
                    _index,
                    instruction_error,
                ) = transaction_error
                {
                    if let solana_program::instruction::InstructionError::Custom(code) =
                        instruction_error
                    {
                        return Err(code);
                    }
                }
            }
            return Err(2);
        }

        self.update_state().await.unwrap();
        Ok(())
    }

    ///read & deserialize account data
    pub async fn get_account_data<T: AccountDeserialize>(&mut self, account: &Pubkey) -> T {
        get_account_data(&mut self.context.banks_client, account).await
    }

    /// Returns the SPL token balance of `token`.
    pub async fn get_token_balance(&mut self, token_account_address: &Pubkey) -> u64 {
        let token_account = self
            .context
            .banks_client
            .get_account(*token_account_address)
            .await
            .unwrap();
        let account_info = TokenAccount::unpack_from_slice(
            token_account
                .expect("token account does not exists")
                .data
                .as_slice(),
        )
        .unwrap();
        account_info.amount
    }
    /// Returns the SPL token balance of `token` or 0 is the account does not exists
    pub async fn get_token_balance_or_zero(&mut self, token_account_address: &Pubkey) -> u64 {
        let token_account = self
            .context
            .banks_client
            .get_account(*token_account_address)
            .await
            .unwrap();
        if token_account.is_none() {
            return 0;
        }
        let account_info =
            TokenAccount::unpack_from_slice(token_account.unwrap().data.as_slice()).unwrap();
        account_info.amount
    }

    /// Returns the SPL token balance of `token`.
    pub async fn get_sol_balance(&mut self, address: &Pubkey) -> u64 {
        self.context
            .banks_client
            .get_balance(*address)
            .await
            .unwrap()
    }

    ///move to slot & show debug info -> returns new clock
    pub async fn move_to_slot(&mut self, new_slot: u64) -> Clock {
        self.context.warp_to_slot(new_slot).unwrap();
        let clock = self.get_clock().await; //get_clock(&mut self.context.banks_client).await.unwrap();
        println!(
            "-- move_to_slot({}), epoch:{} slot:{}",
            new_slot, clock.epoch, clock.slot
        );
        clock
    }

    /// get the cluster clock
    pub async fn get_clock(&mut self) -> Clock {
        get_clock(&mut self.context.banks_client).await.unwrap()
    }

    pub async fn move_to_next_epoch(&mut self) -> Clock {
        //let clock = get_clock(&mut self.context.banks_client).await.unwrap();
        let clock = self.get_clock().await;

        println!("--- current epoch {} slot {}", clock.epoch, clock.slot);
        let next_epoch_start = self
            .context
            .genesis_config()
            .epoch_schedule
            .get_first_slot_in_epoch(clock.epoch + 1);
        return self.move_to_slot(next_epoch_start).await;
    }
    /// Create a user account with some SOL
    /// it executes the tx (it does not add another instruction to transaction builder)
    pub async fn create_test_user_from_keypair(
        &mut self,
        name: &str,
        lamports: u64,
        keypair: Keypair,
    ) -> TestUser {
        println!(
            "--- creating user {} with {} SOL",
            name,
            lamports_to_sol(lamports)
        );
        let new_user = TestUser {
            name: String::from(name),
            keypair: Arc::new(keypair),
        };
        // transfer sol to new pubkey
        self.builder
            .transfer_lamports(
                self.fee_payer_signer(),
                &new_user.keypair.pubkey(),
                lamports,
                "fee payer",
                name,
            )
            .unwrap();
        // create the user now
        self.execute().await;

        return new_user;
    }

    /// Create a user account with some SOL
    /// it executes the tx (it does not add another instruction to transaction builder)
    pub async fn create_test_user(&mut self, name: &str, lamports: u64) -> TestUser {
        return self
            .create_test_user_from_keypair(name, lamports, Keypair::new())
            .await;
        // println!(
        //     "--- creating user {} with {} SOL",
        //     name,
        //     lamports_to_sol(lamports)
        // );
        // let new_user = TestUser {
        //     name: String::from(name),
        //     keypair: Arc::new(Keypair::new()),
        // };
        // // transfer sol to new pubkey
        // self.builder
        //     .transfer_lamports(
        //         self.fee_payer_signer(),
        //         &new_user.keypair.pubkey(),
        //         lamports,
        //         "fee payer",
        //         name,
        //     )
        //     .unwrap();
        // // create the user now
        // self.execute().await;

        // return new_user;
    }

    pub fn mint_from_symbol(&mut self, symbol: &str) -> &Pubkey {
        match symbol {
            "mSOL" => &self.state.msol_mint,
            "mSOL-SOL-LP" => &self.state.liq_pool.lp_mint,
            _ => panic!("unknown symbol {}", symbol),
        }
    }

    /// Creates an associated token account for some user
    /// (only adds the instruction, do not executes)
    pub async fn get_or_create_associated_token_account(
        &mut self,
        user: &TestUser,
        symbol: &str,
    ) -> Pubkey {
        let mint = self.mint_from_symbol(symbol);

        let account = get_associated_token_address(&user.keypair.pubkey(), mint);
        match self
            .context
            .banks_client
            .get_account(account)
            .await
            .unwrap()
        {
            Some(account) => {
                println!("Using existing associated {} account {:?}", symbol, account);
            }
            None => {
                println!(
                    "Creating associated {} account {:?} for {}",
                    symbol,
                    account,
                    user.keypair.pubkey()
                );
                let actual_account =
                    self.create_associated_token_account_instruction(&user, symbol);
                assert_eq!(actual_account, account);
            }
        };
        account
    }

    /// Creates an associated token account for some user
    /// (only adds the instruction, do not executes)
    pub fn create_associated_token_account_instruction(
        &mut self,
        user: &TestUser,
        symbol: &str,
    ) -> Pubkey {
        let mint = match symbol {
            "mSOL" => &self.state.msol_mint,
            "mSOL-SOL-LP" => &self.state.liq_pool.lp_mint,
            _ => panic!("unknown symbol {}", symbol),
        };
        self.builder
            .create_associated_token_account(&user.keypair.pubkey(), mint, "user mSOL")
            .unwrap()
    }

    pub async fn show_user_balance(&mut self, user: &TestUser, label: &str) -> u64 {
        let balance = self.get_sol_balance(&user.keypair.pubkey()).await;
        println!(
            "{} balance {}: {} SOL ({})",
            user.name,
            label,
            lamports_to_sol(balance),
            user.keypair.pubkey()
        );
        return balance;
    }

    pub async fn show_token_balance(
        &mut self,
        token_account: &TestTokenAccount,
        label: &str,
    ) -> u64 {
        let balance = self.get_token_balance(&token_account.pubkey).await;
        println!(
            "{}'s {} balance {}: {} {} ({})",
            token_account.user_name,
            token_account.symbol,
            label,
            lamports_to_sol(balance),
            token_account.symbol,
            token_account.pubkey
        );
        balance
    }

    pub fn install_validator(&mut self, validator: Arc<Keypair>, vote: Arc<Keypair>) {
        self.builder.begin();
        self.builder
            .create_account(
                validator.clone(),
                0,
                &system_program::ID,
                &self.rent,
                "Validator account",
            )
            .unwrap();
        self.builder.add_signer(vote.clone());
        for instruction in vote_instruction::create_account(
            &self.fee_payer(),
            &vote.pubkey(),
            &VoteInit {
                node_pubkey: validator.pubkey(),
                authorized_voter: validator.pubkey(),
                ..VoteInit::default()
            },
            self.rent.minimum_balance(VoteState::size_of()),
        ) {
            self.builder
                .add_instruction(instruction, format!("create vote {}", vote.pubkey()))
                .unwrap();
        }
        self.builder.commit();
    }

    pub fn add_validator(&mut self, validator: Arc<Keypair>, vote: Arc<Keypair>, score: u32) {
        self.install_validator(validator, vote.clone());
        self.builder
            .add_validator(
                &self.state,
                self.validator_manager_authority.clone(),
                vote.pubkey(),
                score,
                self.fee_payer_signer(),
            )
            .unwrap();
    }

    pub fn create_stake(&mut self, vote: &Pubkey, lamports: u64, stake: Arc<Keypair>) {
        self.builder.add_signer(stake.clone());
        self.builder
            .add_instruction(
                system_instruction::create_account(
                    &self.fee_payer(),
                    &stake.pubkey(),
                    lamports,
                    std::mem::size_of::<StakeState>() as u64,
                    &stake::program::ID,
                ),
                format!("create stake {}", stake.pubkey()),
            )
            .unwrap();
        self.builder
            .add_instruction(
                stake::instruction::initialize(
                    &stake.pubkey(),
                    &Authorized {
                        staker: self.fee_payer(),
                        withdrawer: self.fee_payer(),
                    },
                    &Lockup::default(),
                ),
                format!("Initialize stake {}", stake.pubkey()),
            )
            .unwrap();
        self.builder
            .add_instruction(
                stake::instruction::delegate_stake(&stake.pubkey(), &self.fee_payer(), vote),
                format!("delegate stake {}", stake.pubkey()),
            )
            .unwrap()
    }

    pub async fn wait_for_stake_transition(&mut self, stake_address: Pubkey) -> anyhow::Result<()> {
        let epoch_schedule = self.context.genesis_config().epoch_schedule;
        loop {
            let clock = self.get_clock().await;
            let stake_history = self
                .context
                .banks_client
                .get_sysvar::<StakeHistory>()
                .await?;
            let stake_data = self
                .context
                .banks_client
                .get_account(stake_address)
                .await?
                .ok_or_else(|| anyhow!("Con not find account {}", stake_address))?
                .data;
            let stake_state = bincode::deserialize::<StakeState>(&stake_data)?;
            let delegation = stake_state
                .delegation()
                .ok_or_else(|| anyhow!("Undelegated stake {}", stake_address))?;
            let (_effective, activating, deactivating) =
                delegation.stake_activating_and_deactivating(clock.epoch, Some(&stake_history));
            if activating == 0 && deactivating == 0 {
                break Ok(());
            } else {
                self.move_to_slot(epoch_schedule.get_first_slot_in_epoch(clock.epoch + 1))
                    .await;
            }
        }
    }

    pub async fn add_test_validators(&mut self) {
        println!("-------- add_test_validators");
        for n in 1..=4 {
            let v = TestValidator::new(&format!("Validator-{}", n));
            self.add_validator(
                v.keypair.clone(),
                v.vote_keypair.clone(),
                /*score*/ 50_000 + 10_000 * n,
            );
            self.execute().await;
            println!("installed & added {}", v.name);
            self.test_validators.push(v);
        }
    }

    pub async fn create_activated_stake_account(
        &mut self,
        vote_pubkey: &Pubkey,
        lamports: u64,
    ) -> Arc<Keypair> {
        //
        let stake_keypair = Arc::new(Keypair::new());
        // create the account
        self.create_stake(vote_pubkey, lamports, stake_keypair.clone());
        self.execute().await;
        self.context
            .increment_vote_account_credits(vote_pubkey, 1000);
        self.move_to_next_epoch().await;
        self.context
            .increment_vote_account_credits(vote_pubkey, 2000);
        self.move_to_next_epoch().await;
        return stake_keypair;
    }
}

//-- HELPER Fns

///read & deserialize account data
pub async fn get_account_data<T: AccountDeserialize>(
    banks_client: &mut BanksClient,
    account: &Pubkey,
) -> T {
    let result: T = AccountDeserialize::try_deserialize(
        &mut banks_client
            .get_account(*account)
            .await
            .unwrap()
            .unwrap()
            .data
            .as_slice(),
    )
    .unwrap();
    result
}

/// Return the cluster clock
pub fn get_clock(banks_client: &mut BanksClient) -> impl Future<Output = io::Result<Clock>> + '_ {
    banks_client.get_account(sysvar::clock::id()).map(|result| {
        let clock_sysvar = result?
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Clock sysvar not present"))?;
        from_account::<Clock, _>(&clock_sysvar).ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Failed to deserialize Clock sysvar")
        })
    })
}

// returns a random lamports amount between from_sol..to_sol
pub fn random_amount(from_sol: u64, to_sol: u64, rng: &mut impl RngCore) -> u64 {
    Uniform::from((from_sol * LAMPORTS_PER_SOL)..(to_sol * LAMPORTS_PER_SOL)).sample(rng)
}

// --------------------------------
// -- INITIALIZE MARINADE REFERRAL
// --------------------------------
//
pub struct MarinadeReferralTestGlobals {
    admin_keypair: Keypair,
    state_key: Pubkey,
    referral_key: Pubkey,
    partner: TestUser,
}

// initialize marinade-referral global state & a referral-account
pub async fn init_marinade_referral_test_globals(
    test: &mut IntegrationTest,
) -> MarinadeReferralTestGlobals {
    // admin_key = AMMK9YLj8PRRG4K9DUsTNPZAZXeVbHiQJxakuVuvSKrn
    const admin_private_key_bytes: [u8; 64] = [
        136, 60, 191, 232, 11, 20, 1, 82, 147, 185, 119, 92, 226, 212, 217, 227, 38, 100, 72, 135,
        189, 121, 32, 38, 93, 10, 41, 104, 38, 158, 171, 38, 138, 239, 196, 48, 200, 45, 19, 235,
        223, 73, 101, 62, 195, 45, 48, 246, 226, 240, 177, 172, 213, 0, 184, 113, 158, 176, 17,
        177, 2, 215, 168, 135,
    ];
    let admin_key: Keypair = Keypair::from_bytes(&admin_private_key_bytes).unwrap();

    let mut admin = test
        .create_test_user_from_keypair(
            "test_referral_admin_user",
            200 * LAMPORTS_PER_SOL,
            admin_key,
        )
        .await;

    let mut partner = test
        .create_test_user("test_referral_partner", 200 * LAMPORTS_PER_SOL)
        .await;

    // referral global state PDA & bump
    // let (global_state_pda, global_state_bump) = Pubkey::find_program_address(
    //     &[GLOBAL_STATE_SEED],
    //     &marinade_referral::marinade_referral::ID
    //     );

    // init global state
    let global_state = Arc::new(Keypair::new());
    let state_key = global_state.pubkey();
    let state_space = 8 + std::mem::size_of::<marinade_referral::states::GlobalState>();
    test.builder.add_signer(global_state); // need to sign with private key to create account
    test.builder
        .add_instruction(
            system_instruction::create_account(
                &test.builder.fee_payer(),
                &state_key,
                test.rent.minimum_balance(state_space),
                state_space as u64,
                &marinade_referral::marinade_referral::ID,
            ),
            format!("pre-create marinade-referral global state acc because banks-clients do not support creation from program {}", state_key),
        )
        .unwrap();
    println!("create global state account");
    test.execute().await;

    {
        let accounts = marinade_referral::accounts::Initialize {
            admin_account: admin.keypair.pubkey(),
            payment_mint: test.state.msol_mint,
            global_state: state_key,
            system_program: system_program::ID,
        };
        let ix_data = marinade_referral::instruction::Initialize {};
        let instruction = Instruction {
            program_id: marinade_referral::marinade_referral::ID,
            accounts: accounts.to_account_metas(None),
            data: ix_data.data(),
        };
        println!("Init global state");
        test.execute_instruction(
            instruction,
            vec![test.fee_payer_signer(), admin.keypair.clone()],
        )
        .await;
        //}
        //let tx = Transaction::new_signed_with_payer(&[deposit_instruction], Some(&user.keypair.pubkey()),&[user.keypair.as_ref()],test.recent_blockhash().await);
        //let tx = Transaction::new_with_payer(&[instruction], Some(&user.keypair.pubkey()));
        // marinade-referral execution
        //test.execute_txn(tx, vec!(user.keypair.clone())).await;
    }

    // partner token account
    let token_partner_account = partner.get_or_create_msol_account_instruction(test).await;
    test.execute().await; // execute if the ATA needed to be created

    // ----------------------
    // create account and init referral state
    // ----------------------
    let referral_state = Arc::new(Keypair::new());
    let referral_key = referral_state.pubkey();
    // 8=Anchor sha-struct-ident, 10 partner-name string
    let referral_state_size =
        8 + 10 + std::mem::size_of::<marinade_referral::states::ReferralState>();
    test.builder.add_signer(referral_state); // need to sign with private key to create account
    test.builder
        .add_instruction(
            system_instruction::create_account(
                &test.builder.fee_payer(),
                &referral_key,
                test.rent.minimum_balance(referral_state_size),
                referral_state_size as u64,
                &marinade_referral::marinade_referral::ID,
            ),
            format!("pre-create referral-state because banks-clients do not support creation from program {}", referral_key),
        )
        .unwrap();
    println!(
        "create referral-state account. msol-mint {}",
        &test.state.msol_mint
    );
    test.execute().await;

    {
        let accounts = marinade_referral::accounts::InitReferralAccount {
            global_state: state_key,
            admin_account: admin.keypair.pubkey(),
            referral_state: referral_key,
            partner_account: partner.keypair.pubkey(),
            payment_mint: test.state.msol_mint,
            token_partner_account: token_partner_account.pubkey,
            system_program: system_program::ID,
            token_program: spl_token::ID,
            rent: solana_sdk::sysvar::rent::ID,
        };
        //let partner_name: [u8;10] = ascii::ascii_char::AsciiChar = "TEST_PART";
        let ix_data = marinade_referral::instruction::InitReferralAccount {
            partner_name: "TEST_PART".into(),
        };
        let instruction = Instruction {
            program_id: marinade_referral::marinade_referral::ID,
            accounts: accounts.to_account_metas(None),
            data: ix_data.data(),
        };
        println!("Init referral-state");
        test.execute_instruction(
            instruction,
            vec![test.fee_payer_signer(), admin.keypair.clone()],
        )
        .await;
    }

    return MarinadeReferralTestGlobals {
        admin_keypair: Keypair::from_bytes(&admin_private_key_bytes).unwrap(),
        state_key,
        referral_key,
        partner,
    };
}
