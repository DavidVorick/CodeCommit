#!/bin/bash

# A script to aggregate specified project files into a single text file named
# 'codeRollup.txt' that can be provided as context to an LLM.  Any files
# located in folders named 'assets', appearing at any depth of the file
# hierarchy, will be skipped.
#
# Usage:
#   ./codeRollup.sh
#   ./codeRollup.sh [component1] [component2] ...
#   ./codeRollup.sh --all
#
# A "component" is a subdirectory within the 'src/' directory.

OUTPUT_FILE="agent-config/codeRollup.txt"
components=()
process_all=false

exec > "$OUTPUT_FILE"

for arg in "$@"; do
    if [[ "$arg" == "--all" ]]; then
        process_all=true
    else
        components+=("$arg")
    fi
done

if [[ "$process_all" == true && ${#components[@]} -gt 0 ]]; then
    echo "Error: --all flag is present. Please do not also provide components: ${components[*]}" >&2
    rm "$OUTPUT_FILE"
    exit 1
fi

print_file_with_header() {
    local file_path="$1"

    if [[ "$file_path" =~ (^|/)assets(/|$) ]]; then
        return
    fi

    if [ -f "$file_path" ]; then
        echo "--- $file_path ---"
        cat "$file_path"
        echo
    else
        echo "Error: File '$file_path' not found, exiting." >&2
        rm "$OUTPUT_FILE"
        exit 1
    fi
}

process_directory() {
    local dir_path="$1"
    if [ -d "$dir_path" ]; then
        if [[ "$(basename "$dir_path")" == "assets" ]]; then
            return
        fi
        find "$dir_path" -maxdepth 1 -type f | sort | while read -r file; do
            print_file_with_header "$file"
        done
    fi
}

process_directory_recursive() {
    local dir_path="$1"
    if [ -d "$dir_path" ]; then
        find "$dir_path" -type d -name assets -prune -o -type f -print | sort | while read -r file; do
            print_file_with_header "$file"
        done
    fi
}

root_files=(
    "Cargo.toml"
    "UserSpecification.md"
    "LLMInstructions.md"
    "build.sh"
    "codeRollup.sh"
    ".gitignore"
)
for file in "${root_files[@]}"; do
    print_file_with_header "$file"
done

if [[ "$process_all" == true ]]; then
    process_directory_recursive "src"
else
    process_directory "src"
    for component in "${components[@]}"; do
        component_path="src/$component"
        if [ -d "$component_path" ]; then
            process_directory "$component_path"
        else
            echo "Error: Component directory '$component_path' not found. Exiting." >&2
            rm "$OUTPUT_FILE"
            exit 1
        fi
    done
fi

if [ -d "tests" ]; then
    process_directory_recursive "tests"
fi

echo "Code rollup complete. Output is in '$OUTPUT_FILE'" >&2
