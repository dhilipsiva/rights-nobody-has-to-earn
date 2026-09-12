#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Build incrementally, then run every substantive pin and contradiction check.

set -euo pipefail
cd -- "$(dirname -- "$0")"

cargo build --release --locked --quiet --bin rights-verify
exec target/release/rights-verify "$@"
