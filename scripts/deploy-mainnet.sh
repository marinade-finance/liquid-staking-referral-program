#!/bin/bash
set -ex
echo "deploy mainnet AFFECTING MAINNET USERS?"
read -p "Press any key to continue..."
echo "building VERIFIABLE .so"
anchor build --verifiable
solana program deploy -v -u http://marinade.rpcpool.com \
    target/verifiable/marinade_referral.so \
    --program-id target/deploy/marinade_referral-keypair.json \
    --upgrade-authority ~/.config/solana/MRAX9jRu8i6Gx63t2e8XmKHSJmop4bgveTwnES8TSU3.json
