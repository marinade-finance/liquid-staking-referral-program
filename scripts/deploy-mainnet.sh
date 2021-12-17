#!/bin/bash
set -e
echo "deploy mainnet AFFECTING MAINNET USERS?"
read -p "Press any key to continue..."
echo "building VERIFIABLE .so"
anchor build --verifiable
solana program deploy -v -u mainnet \
    target/verifiable/marinade_referral.so \
    --program-id target/deploy/marinade_referral-keypair.json \
    --upgrade-authority target/deploy/marinade_referral-keypair.json
