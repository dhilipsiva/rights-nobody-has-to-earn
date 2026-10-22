#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Put the pinned Nibli engine beside this checkout so ./verify.sh can build.
#
# With git-lfs installed (and `git lfs install` run once), from clean inputs this is
# the whole setup:
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
if [ "$(head -c 42 "$constitution" 2>/dev/null)" = "version https://git-lfs.github.com/spec/v1" ]; then
    cat >&2 <<MESSAGE
$constitution is a Git LFS pointer ($(wc -c < "$constitution") bytes), not the
constitution: this checkout was made without git-lfs. Install git-lfs, then run

    git lfs install && git lfs pull

MESSAGE
    # The only copy outside LFS storage is the text as it stood when it moved to LFS
    # (3c46a347). It is the right file only while the pointer still names that object.
    if grep -qx "oid sha256:73fbc8c86df7ac0d4174367001a18d77723e30cde6a5ce40ecb4038136a26ecb" "$constitution"; then
        cat >&2 <<MESSAGE
Without git-lfs, restore the identical file (the same sha256) from the commit
before the move, from the repository root:

    git restore --source=3c46a347 -- $constitution

git status then lists it as modified, which is expected without git-lfs. Do not
commit it: that would replace the LFS pointer with the 57 MB file.
MESSAGE
    else
        echo "This pointer names a later version, which only LFS storage holds: git-lfs is required." >&2
    fi
    exit 1
fi

pin_field() {
    awk -v key="$1" '$1 == key { print $2; found = 1 } END { exit !found }' engine.pin
}

repository="$(pin_field repository)"
revision="$(pin_field revision)"

# The build reaches the engine through ./engine (the Cargo path dependencies), so the engine can
# live anywhere. RIGHTS_ENGINE_PATH chooses it, for example a separate clone kept at the pin
# rather than a working checkout; otherwise an existing link stays, and the default is ../nibli.
if [ -n "${RIGHTS_ENGINE_PATH:-}" ]; then
    engine="$RIGHTS_ENGINE_PATH"
elif [ -L engine ]; then
    engine="$(readlink engine)"
else
    engine=../nibli
fi
if [ -e engine ] && [ ! -L engine ]; then
    echo "./engine exists and is not a link; move it aside first" >&2
    exit 1
fi

echo "pinned engine: $revision"
echo "               $repository"

if [ ! -e "$engine/.git" ]; then
    echo "cloning into $engine"
    git clone --quiet "$repository" "$engine"
    git -C "$engine" checkout --quiet --detach "$revision"
    echo "checked out $revision (detached)"
fi
ln -sfn "$(cd -- "$engine" && pwd -P)" engine
echo "engine link: engine -> $(readlink engine)"

present="$(git -C engine rev-parse HEAD)"
echo "present engine: $present ($engine)"

if [ -n "$(git -C engine status --porcelain)" ]; then
    echo "NOTE: that checkout has uncommitted changes; ./verify.sh will build them."
fi

if [ "$present" = "$revision" ]; then
    echo "matches the pin; run ./verify.sh"
    exit 0
fi

cat <<MESSAGE
NOTE: the engine beside this checkout is NOT the pinned revision. Nothing here
changes it. ./verify.sh will build what is there, and a measurement taken now
describes that engine, not the pinned one. To verify against the pin instead, keep
a separate clone at the pin and link it (this checkout's engine is left alone):

    RIGHTS_ENGINE_PATH=../nibli-at-pin ./bootstrap.sh
MESSAGE
exit 0
