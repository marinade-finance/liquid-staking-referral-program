# liquid-staking-referral-program
Wrapper functions over the liquid-staking-program main stake/unstake functions. Allows referrals from partners providing Marinade liquid-staking as a service to their users

Design doc: https://docs.google.com/document/d/1aXq3oEBF-cAXJpF_ubteaI-4oYV8wkk2E8hdTRjti2g/edit#heading=h.xaz348hsh3eq (Google doc, request access please)

## Installation
```bash
yarn install
```

## Build program
```bash
anchor build
```

## Integration Tests (separate project)
```bash
bash test.sh
```

## Test program
```bash
anchor test
```

## Deploy an new copy of the program on a random address
```bash
anchor deploy --provider.cluster devnet
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

## Custom types that should be manually injected to idl.json
```json
{
// ...
  "types": [
    // ...,
    {
      "name": "Fee",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "basisPoints",
            "type": "u32"
          }
        ]
      }
    }
  ],
// ...
}
```
