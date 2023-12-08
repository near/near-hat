#!/bin/bash
set -e
cd "$(dirname $0)"
cargo build --target wasm32-unknown-unknown --release
