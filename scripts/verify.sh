#!/bin/bash
set -ex
cd programs/marinade-referral
anchor verify mRefx8ypXNxE59NhoBqwqb3vTvjgf8MYECp4kgJWiDY --provider.cluster mainnet
cd ../..
