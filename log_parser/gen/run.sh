#!/usr/bin/env bash

set -e

rm -rf ./out/*

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PYTHONPATH="$ROOT${PYTHONPATH:+:$PYTHONPATH}"

for file in ./*.py ./tex/*.py; do
    # Skip if no files match (avoids literal pattern issues)
    [ -e "$file" ] || continue

    case "$file" in
        ./setup.py|./test.py)
            continue
            ;;
    esac

    echo "Running $file"
    python "$file" > /dev/null
done