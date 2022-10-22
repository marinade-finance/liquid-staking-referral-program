#!/bin/bash
set -ex
echo "upgrade devnet AFFECTING DEVNET USERS?"
read -p "Press any key to continue..."
anchor upgrade --program-id MR2LqxoSbw831bNy68utpu5n4YqBH3AzDmddkgk9LQv \
   --provider.cluster devnet ./target/deploy/marinade_referral.so \
   --provider.wallet ~/.config/solana/3Pb4Q6XcZCCgz7Gvd229YzFoU1DpQ4myUQFx8Z9AauQ6.json
