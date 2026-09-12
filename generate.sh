#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
set -euo pipefail
cd -- "$(dirname -- "$0")"
cargo build --release --locked --quiet --bin generate
exec target/release/generate "$@"
