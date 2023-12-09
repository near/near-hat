#!/bin/bash

RUST_LOG=info cargo run -p near-hat-cli -- start --contracts-to-spoon usdt.tether-token.near
