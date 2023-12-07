#!/bin/bash

mitmdump --mode regular -p 80 -s dns.py & RUST_LOG=info cargo run -p near-hat-cli -- start