#!/bin/bash
set -ex
echo "upgrade mainnet?"
read -p "Press any key to continue..."
anchor upgrade --program-id MR2LqxoSbw831bNy68utpu5n4YqBH3AzDmddkgk9LQv \
   --provider.cluster mainnet ./target/deploy/marinade_referral.so
