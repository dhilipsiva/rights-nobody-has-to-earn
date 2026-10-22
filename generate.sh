#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
set -euo pipefail
cd -- "$(dirname -- "$0")"
cargo build --release --locked --quiet --bin generate
# Where cargo builds: honours CARGO_TARGET_DIR and build.target-dir.
target_dir="$(cargo metadata --format-version 1 --no-deps --locked | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
exec "${target_dir:-target}/release/generate" "$@"
