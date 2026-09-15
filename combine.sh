#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Concatenate the numbered Book 1 chapters, and their pin files, for reading in
# one pass. It writes only the two combined outputs and verifies nothing.

find book-1 -maxdepth 1 -type f -name "*.md" | grep -E '/0[1-9]-|/1[0-5]-' | sort | while IFS= read -r file; do 
    cat "$file"
    printf "\n\n---\n\n"
done > combined.md
find book-1 -maxdepth 1 -type f -name "*.pins.nibli" | grep -E '/0[1-9]-|/1[0-5]-' | sort | while IFS= read -r file; do 
    cat "$file"
    printf "\n\n---\n\n"
done > combined.pins.nibli.md
