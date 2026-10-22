#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Put the pinned Nibli engine beside this checkout so ./verify.sh can build.
#
# From clean inputs this is the whole setup:
#
#     git clone https://github.com/dhilipsiva/rights-nobody-has-to-earn.git
#     cd rights-nobody-has-to-earn
#     ./bootstrap.sh
#     ./verify.sh
#
# It never edits an existing engine checkout. If one is already there it says
# what revision that is and whether it matches `engine.pin`, and leaves the
# decision to you — the engine is developed alongside this book, and a script
# that silently reset someone's work in progress would be worse than a mismatch.

set -euo pipefail
cd -- "$(dirname -- "$0")"

# The constitution is stored with Git LFS. Without git-lfs a clone holds a 133-byte pointer in its
# place, which nothing here can verify; stop and say so rather than fail later in the parser.
constitution=book-1/source/constitution.nibli
if [ "$(head -c 42 "$constitution")" = "version https://git-lfs.github.com/spec/v1" ]; then
    cat >&2 <<MESSAGE
$constitution is a Git LFS pointer ($(wc -c < "$constitution") bytes), not the
constitution: this checkout was made without git-lfs. Install git-lfs, then run

    git lfs install && git lfs pull

Without git-lfs, restore the identical file (same sha256) from the commit before
it moved to LFS:

    git show 3c46a347:$constitution > $constitution

git status then reports it as modified, which is expected without git-lfs.
MESSAGE
    exit 1
fi

pin_field() {
    awk -v key="$1" '$1 == key { print $2; found = 1 } END { exit !found }' engine.pin
}

repository="$(pin_field repository)"
revision="$(pin_field revision)"
engine="${RIGHTS_ENGINE_PATH:-../nibli}"

echo "pinned engine: $revision"
echo "               $repository"

if [ ! -d "$engine/.git" ]; then
    echo "cloning into $engine"
    git clone --quiet "$repository" "$engine"
    git -C "$engine" checkout --quiet --detach "$revision"
    echo "checked out $revision (detached)"
    echo "now run ./verify.sh"
    exit 0
fi

present="$(git -C "$engine" rev-parse HEAD)"
echo "present engine: $present ($engine)"

if [ -n "$(git -C "$engine" status --porcelain)" ]; then
    echo "NOTE: that checkout has uncommitted changes; ./verify.sh will build them."
fi

if [ "$present" = "$revision" ]; then
    echo "matches the pin; run ./verify.sh"
    exit 0
fi

cat <<MESSAGE
NOTE: the engine beside this checkout is NOT the pinned revision. Nothing here
changes it. ./verify.sh will build what is there, and a measurement taken now
describes that engine, not the pinned one. To verify against the pin instead:

    git -C "$engine" fetch origin
    git -C "$engine" checkout --detach $revision
MESSAGE
exit 0
