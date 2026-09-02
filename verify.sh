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

# The two build stamps feed the run-diagnostics module only. They are not in
# the receipt's sanitized-environment allowlist, so no receipt binds them.
RIGHTS_VERIFY_BUILD_STARTED="${EPOCHREALTIME:-}"
cargo build --release --locked --quiet --bin rights-verify
RIGHTS_VERIFY_BUILD_FINISHED="${EPOCHREALTIME:-}"
export RIGHTS_VERIFY_BUILD_STARTED RIGHTS_VERIFY_BUILD_FINISHED
exec target/release/rights-verify "$@"
