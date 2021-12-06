clear
export RUST_LOG=solana_runtime::system_instruction_processor=trace,solana_runtime::message_processor=debug,solana_bpf_loader=debug,solana_rbpf=debug
#export RUST_LOG=solana_metrics=info,debug
#cargo test delayed_unstake --manifest-path programs/marinade-finance/tests/Cargo.toml -- --nocapture 
#cargo test set_lp_params --manifest-path programs/marinade-finance/tests/Cargo.toml -- --nocapture 
#cargo test test_add_liquidity --manifest-path programs/marinade-finance/tests/Cargo.toml -- --nocapture 
#cargo test test_deposit_sol --manifest-path programs/marinade-finance/tests/Cargo.toml -- --nocapture 
#cargo test test_add_validator --manifest-path programs/marinade-finance/tests/Cargo.toml -- --nocapture 
#cargo test merge --manifest-path programs/marinade-finance/tests/Cargo.toml -- --nocapture 
#cargo test test_config_validator_system --manifest-path programs/marinade-finance/tests/Cargo.toml -- --nocapture 
#cargo test unstake_unlisted --manifest-path programs/marinade-finance/tests/Cargo.toml 
#cargo test --manifest-path programs/marinade-finance/tests/Cargo.toml -- --nocapture 
RUSTFLAGS=-Awarnings cargo +nightly test --manifest-path programs/marinade-referral/tests/Cargo.toml
