#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Build freshness remains part of every invocation: Cargo's incremental check is
# effectively free when neither this verifier nor the embedded Nibli engine has
# changed. All validation, execution, locking, receipts, and commit gates then
# run inside one native process.
#
# Reviewed-reference compatibility anchor: 4d. a control must not pollute the base below it
# The executable guard now lives in `src/checks/repository.rs::check_control_scopes`.

set -euo pipefail
cd -- "$(dirname -- "$0")"

cargo build --release --locked --quiet --bin rights-verify
exec target/release/rights-verify "$@"
