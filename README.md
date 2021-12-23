# liquid-staking-referral-program
Wrapper functions over Marinade's liquid-staking-program main stake/unstake functions. Allows referrals from partners providing Marinade liquid-staking as a service to their users, getting a share fo rewards.

Documentation: 
https://docs.marinade.finance/partnerships/referral-program

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

## Upgrade program
```bash
anchor upgrade --program-id mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY --provider.cluster devnet ./target/deploy/marinade_referral.so --provider.wallet ~/.config/solana/mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY.json
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
