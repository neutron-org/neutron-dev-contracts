.PHONY: schema test clippy build fmt compile check_contracts

schema:
	@find contracts/* -maxdepth 0 -type d \( ! -name . \) -exec bash -c "cd '{}' && cargo schema" \;
test:
	@cargo test

clippy:
	@cargo clippy --all --all-targets -- -D warnings

fmt:
	@cargo fmt -- --check

compile:
	@docker run --rm -v "$(CURDIR)":/code \
	    --mount type=volume,source="$(notdir $(CURDIR))_cache",target=/target \
	    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	    --platform linux/amd64 \
	    cosmwasm/workspace-optimizer:0.16.0

check_contracts:
	@cargo install cosmwasm-check --version 2.0.4 --locked
	@cosmwasm-check --available-capabilities iterator,staking,stargate,neutron,cosmwasm_1_1,cosmwasm_1_2,cosmwasm_1_3,cosmwasm_1_4,cosmwasm_2_0 artifacts/*.wasm

build: schema clippy test fmt compile check_contracts
