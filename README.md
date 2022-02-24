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


## start relaygenerate two chain `wasm` and `genesis state` 
```
./target/release/parachain-collator export-genesis-wasm --chain rococo-local-parachain-2000-raw.json > para-2000-wasm
./target/release/parachain-collator export-genesis-wasm --chain rococo-local-parachain-3000-raw.json > para-3000-wasm
./target/release/parachain-collator export-genesis-state --chain rococo-local-parachain-2000-raw.json > para-2000-genesis
./target/release/parachain-collator export-genesis-state --chain rococo-local-parachain-3000-raw.json > para-3000-genesis
```

## start Parachain:2000
```
./target/release/parachain-collator \ 
--alice \
--collator \
--force-authoring \
--chain rococo-local-parachain-2000-raw.json \
--base-path /tmp/parachain/alice \
--port 40333 \
--ws-port 8844 \
-- \
--execution wasm \
--chain ../../polkadot/rococo-local-cfde.json \  
--port 30343 \
--ws-port 9977
```

## start Parachain:3000
```
./target/release/parachain-collator \ 
--bob \
--collator \
--force-authoring \
--chain rococo-local-parachain-3000-raw.json \
--base-path /tmp/parachain/bob \
--port 40334 \
--ws-port 8855 \
-- \
--execution wasm \
--chain ../../polkadot/rococo-local-cfde.json \  
--port 30344 \
--ws-port 9988
```