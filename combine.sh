#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Concatenate the numbered Book 1 chapters, and their pin files, for reading in
# one pass. It writes only the two combined outputs and verifies nothing.

# The numbered chapters, opening note excluded. The prefix is the manifest's
# position (book-1/contents.json), which is what makes the sort the order.
find book-1 -maxdepth 1 -type f -name "*.md" | grep -E '/[0-9]{2}-' | grep -v '/00-' | sort | while IFS= read -r file; do 
    cat "$file"
    printf "\n\n---\n\n"
done > combined.md
find book-1 -maxdepth 1 -type f -name "*.pins.nibli" | grep -E '/[0-9]{2}-' | grep -v '/00-' | sort | while IFS= read -r file; do 
    cat "$file"
    printf "\n\n---\n\n"
done > combined.pins.nibli.md
