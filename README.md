# Marinade Liquid Saking Referral Program
Wrapper functions over Marinade's liquid-staking-program main stake/unstake functions. Allows referrals from partners providing Marinade liquid-staking as a service to their users, getting a share of rewards.

Documentation: 
https://docs.marinade.finance/partnerships/referral-program

## This program interacts with Marinade via CPI calls

This program is also a good example on how to integrate Marinade from another on-chain program

Use this program as an example on how to read Marinade data and use Marinade instructions from another on-chain program

## Examples from this code

### Get true mSOL/SOL price

Frist you need to read Marinade state, example here:
https://github.com/marinade-finance/liquid-staking-referral-program/blob/main/programs/marinade-referral/src/instructions/liquid_unstake.rs#L42

After reading Marinade state, you'll have `marinade_state.msol_price: u64`, and that's mSOL price in SOL multiplied by 0x1_0000_0000 (shifted), so to obtain mSOL/SOL as f64 you should do: `let msol_price_f64: f64 = marinade_state.msol_price as f64 / 0x1_0000_0000 as f64`, and then you get the true mSOL/SOL price.

### How much SOL an amount of mSOL represents

You start with the previous example and some amount of mSOL-lamports, then:

`let SOL_lamports = (mSOL_lamports as u128 * marinade_state.msol_price as u128 / 0x1_0000_0000 as u128) as u64`

Note: mSOL uses 9 decimals, as SOL.

### How much mSOL an amount of SOL represents

`let mSOL_lamports = (mSOL_lamports as u128 * 0x1_0000_0000 as u128 / marinade_state.msol_price as u128) as u64`

### Derive mSOL/USDC price

If you have access to SOL/USDC price from an oracle, the best way to avoid losess due to price inaccuracies is to derive mSOL/USDC from SOL/USDC and Marinade true price

`let mSOL_usdc = (SOL_usdc as u128 * marinade_state.msol_price as u128 / 0x1_0000_0000 as u128) as u64`

### Notes

msol_price is computed here https://github.com/marinade-finance/liquid-staking-program/blob/main/programs/marinade-finance/src/state/update.rs#L247 after each epoch ends, when SOL staking rewards are added to the pool. You can also use the fns `marinade_state.calc_lamports_from_msol_amount()` and `marinade_state.calc_msol_from_lamports()` for better precision computing mSOL from SOL and viceversa.

### Stake (convert SOL -> mSOL, zero fee)

Example here: https://github.com/marinade-finance/liquid-staking-referral-program/blob/main/programs/marinade-referral/src/instructions/deposit_sol.rs#L9

### Deposit Stake Account (convert SOL in a stake-account -> mSOL, zero fee)

Example here: https://github.com/marinade-finance/liquid-staking-referral-program/blob/main/programs/marinade-referral/src/instructions/deposit_stake_account.rs#L10

### Liquid-Unstake (convert mSOL -> SOL, 0.3 to 3% fee)

Example here: https://github.com/marinade-finance/liquid-staking-referral-program/blob/main/programs/marinade-referral/src/instructions/liquid_unstake.rs#L9


## Installation
```bash
yarn install
```

## Build program
```bash
anchor build
```

## Re-Deploy on devnet address mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY
```bash
bash scripts/deploy-testnet.sh
```

## Devnet Upgrade program
```bash
anchor upgrade --program-id mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY --provider.cluster devnet ./target/deploy/marinade_referral.so --provider.wallet ~/.config/solana/3Pb4Q6XcZCCgz7Gvd229YzFoU1DpQ4myUQFx8Z9AauQ6.json
```

## Autofix TypeScript lint errors
```bash
yarn lint
```

## Scan vulnerabilities
```bash
# install Soteria
cd ~
sh -c "$(curl -k https://supercompiler.xyz/install)"
export PATH=$HOME/soteria-linux-develop/bin/:$PATH
cd -

# check vulnerabilities
cd programs/marinade-referral
# check vulnerabilities in selected library codes
soteria .
# check vulnerabilities in all library codes
soteria -analyzeAll .
```
