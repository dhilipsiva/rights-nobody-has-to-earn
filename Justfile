# Local gates, cut from the portfolio's template (products/templates/Justfile). Run them inside
# `nix develop`, with the Nibli engine beside this checkout (./bootstrap.sh) and the
# constitution fetched through Git LFS.

set shell := ["bash", "-euo", "pipefail", "-c"]

export RUST_BACKTRACE := "1"

# Test threads: each test holds the whole constitution (59 GB at 32 parallel tests, measured).
test_threads := env("RIGHTS_TEST_THREADS", "2")

# List the recipes.
default:
    @just --list

# Type-check the verifier and generator, and the reading interface's engine adapter. The
# interface crate itself embeds generated files: run `python3 ui/scripts/build.py prepare` first.
check:
    cargo check --locked --all-targets
    cargo check --locked --manifest-path ui/Cargo.toml -p book-reason --all-targets

# Formatting and Clippy. Package-scoped: `cargo fmt --all` would also format the engine's
# path dependencies.
lint:
    cargo fmt -p rights-verify -- --check
    cargo clippy --locked --workspace --all-targets -- -D warnings

# The Rust tests, in release, and the Python checks.
test:
    RUST_MIN_STACK=67108864 cargo test --release --locked --workspace --no-fail-fast -- --test-threads={{test_threads}}
    uv run --with markdown-it-py==4.2.0 --with mdit-py-plugins==0.6.1 python -m unittest discover -s tests -p 'test_*.py'
    python3 tools/prose_lint.py --check --locations
    python3 registry/check.py
    python3 tools/assurance_pages.py --check

# Every substantive pin and contradiction check (README, Verify).
verify:
    ./verify.sh

# Assemble the book as HTML, EPUB and PDF (book-1/README.md).
build-book:
    uv run --script tools/build_book.py

# Licences, bans and sources for both Cargo workspaces.
deny:
    cargo deny check licenses bans sources
    cargo deny --manifest-path ui/Cargo.toml check licenses bans sources

# What CI checks.
ci: check lint test deny
