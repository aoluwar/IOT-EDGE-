#!/bin/bash
set -euo pipefail
cd ota/applier
cargo build --release --target aarch64-unknown-linux-gnu
echo "Built applier for aarch64"
