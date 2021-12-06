#!/bin/bash
set -e
echo "deploy devnet AFFECTING DEVNET USERS?"
read -p "Press any key to continue..."
anchor build
#cp target/idl/marinade_program.json res/
solana program deploy -v -u devnet --program-id target/deploy/marinade_referral-keypair.json target/deploy/marinade_referral.so --upgrade-authority target/deploy/marinade_referral-keypair.json
