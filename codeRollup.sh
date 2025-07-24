#!/bin/bash

# A script to aggregate specified project files into a single text file
# named 'codeRollup.txt' for easy copy-pasting as context for an LLM.

# --- Configuration ---
# The final output file. This file will be overwritten on each run.
OUTPUT_FILE="codeRollup.txt"

# --- Main Script ---

# Redirect all standard output from this point forward to the output file.
# Standard error is not redirected, so warnings will still appear in the terminal.
exec > "$OUTPUT_FILE"

# Function to print a file's content with a standardized header
print_file_with_header() {
    local file_path="$1"
    
    # Check if the file exists and is a regular file
    if [ -f "$file_path" ]; then
        echo "=================================================="
        echo "--- File: $file_path"
        echo "=================================================="
        cat "$file_path"
        echo
        echo
    else
        # Print a warning to standard error if a file is not found.
        # This will appear in the terminal, not in the rollup file.
        echo "Warning: File '$file_path' not found, skipping." >&2
    fi
}

# --- File Processing ---

# 1. Process specific files in the root directory
root_files=(
    "Cargo.toml"
    "UserSpecification.md"
    "codeRollup.sh"
    "README.md"
)
for file in "${root_files[@]}"; do
    print_file_with_header "$file"
done

# 2. Process all files in specified subdirectories
scan_dirs=(
    "src"
    "tests"
)
for dir in "${scan_dirs[@]}"; do
    if [ -d "$dir" ]; then
        find "$dir" -type f | sort | while read -r file; do
            print_file_with_header "$file"
        done
    fi
done

# --- Finalization ---

# Print a success message to standard error so the user sees it in the terminal.
echo "Rollup complete. Output written to $OUTPUT_FILE." >&2
