#!/bin/bash
set -ex
echo "deploy devnet AFFECTING DEVNET USERS?"
read -p "Press any key to continue..."
anchor build
solana program deploy -v -u devnet \
    target/deploy/marinade_referral.so\
    --program-id target/deploy/marinade_referral-keypair.json\
    --upgrade-authority target/deploy/marinade_referral-keypair.json
