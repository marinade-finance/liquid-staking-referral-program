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

## Test program
```bash
anchor test
```

## Deploy to devnet
```bash
anchor deploy --provider.cluster devnet
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
cd programs/marinade-referrral
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
