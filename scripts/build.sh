#!/bin/bash
set -e
cd "./contracts"
cargo build --all --target wasm32-unknown-unknown --release
cp ./target/wasm32-unknown-unknown/release/*.wasm ./res/
