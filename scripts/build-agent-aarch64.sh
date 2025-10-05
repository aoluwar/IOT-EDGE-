#!/bin/bash
set -euo pipefail
cargo build --release --target aarch64-unknown-linux-gnu
echo "Built agent for aarch64"
