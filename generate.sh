#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
set -euo pipefail
cd -- "$(dirname -- "$0")"
# The engine the build uses (./engine, linked by bootstrap.sh), named in the output.
if ! revision="$(git -C engine rev-parse HEAD 2>/dev/null)"; then
    echo "no engine at ./engine: run ./bootstrap.sh" >&2
    exit 2
fi
[ -z "$(git -C engine status --porcelain)" ] || revision="$revision, with uncommitted changes"
echo "engine: $revision ($(cd engine && pwd -P))"
cargo build --release --locked --quiet --bin generate
# Where cargo builds: honours CARGO_TARGET_DIR and build.target-dir.
target_dir="$(cargo metadata --format-version 1 --no-deps --locked | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
exec "${target_dir:-target}/release/generate" "$@"
