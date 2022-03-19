# Proof Parachain  :ledger::ledger::ledger::ledger::ledger::ledger:
>For Studying the XCM in polkadot 

### This is a Proof Parachain connected to the rococo test network

This chain version is built with `polkadot-v0.9.16`
Require Rust version:`nightly-2021-11-07-x86_64-unknown-linux-gnu`

## start relaychain
```
./target/release/polkadot --chain rococo-local-cfde.json --alice --tmp --port 30333 --ws-port 9944
./target/release/polkadot --chain rococo-local-cfde.json --bob --tmp --port 30334 --ws-port 9945
./target/release/polkadot --chain rococo-local-cfde.json --dave --tmp --port 30335 --ws-port 9946
```
## generate wasm and genesis state
```

 ./target/release/parachain-Proof build-spec --disable-default-bootnode > rococo-local-parachain-plain.json 

./target/release/parachain-Proof build-spec --chain rococo-local-parachain-plain.json --raw --disable-default-bootnode > rococo-local-parachain-3000-raw.json

 ./target/release/parachain-Proof export-genesis-wasm --chain rococo-local-parachain-3000-raw.json > para-3000-wasm

./target/release/parachain-Proof  export-genesis-state --chain rococo-local-parachain-3000-raw.json > para-3000-genesis


 ./target/release/parachain-Proof \
--alice \
--collator \
--force-authoring \
--chain rococo-local-parachain-3000-raw.json \
--base-path /tmp/parachain/alice \
--port 40333 \
--ws-port 8844 \
-- \
--execution wasm \
--chain ../../polkadot-0.9.16/rococo-local-cfde.json \ 
--port 30343 \
--ws-port 9977
```