#!/bin/sh

ABSPATH=`readlink -f "$0"`
CURRENT_DIR=`dirname "$ABSPATH"`

clear
export RUST_LOG=solana_runtime::system_instruction_processor=trace,solana_runtime::message_processor=debug,solana_bpf_loader=debug,solana_rbpf=debug
#export RUST_LOG=solana_metrics=info,debug

# +nightly does not work in current
RUSTFLAGS=-Awarnings cargo test $@ --manifest-path $CURRENT_DIR/../programs/marinade-referral/tests/Cargo.toml
