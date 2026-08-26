find book-1 -maxdepth 1 -type f -name "*.md" | grep -E '/0[1-9]-|/1[0-5]-' | sort | while IFS= read -r file; do 
    cat "$file"
    printf "\n\n---\n\n"
done > combined.md
find book-1 -maxdepth 1 -type f -name "*.pins.nibli" | grep -E '/0[1-9]-|/1[0-5]-' | sort | while IFS= read -r file; do 
    cat "$file"
    printf "\n\n---\n\n"
done > combined.pins.nibli.md
