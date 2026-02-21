#!/usr/bin/env bash
# validate-site.sh — Verify assembled site before deployment.
set -euo pipefail

SITE="${1:?Usage: validate-site.sh <site-dir>}"
ERRORS=0

err() { echo "ERROR: $*" >&2; ERRORS=$((ERRORS + 1)); }

echo "=== Checking required files ==="
REQUIRED=(
    index.html
    style.css
    main.js
    pkg/rulebound_demo.js
    pkg/rulebound_demo_bg.wasm
)
for f in "${REQUIRED[@]}"; do
    if [[ ! -f "$SITE/$f" ]]; then
        err "Missing required file: $f"
    else
        echo "  OK  $f"
    fi
done

echo "=== Checking local JS imports ==="
for jsfile in "$SITE"/*.js; do
    [[ -f "$jsfile" ]] || continue
    dir=$(dirname "$jsfile")
    imports=$(grep -oE "from ['\"](\./|\.\./)[^'\"]+['\"]" "$jsfile" 2>/dev/null \
        | sed "s/from ['\"]//;s/['\"]$//" || true)
    [[ -z "$imports" ]] && continue
    while IFS= read -r imp; do
        target="$dir/$imp"
        if [[ ! -f "$target" ]]; then
            err "$(basename "$jsfile"): import '$imp' -> file not found ($target)"
        else
            echo "  OK  $(basename "$jsfile"): $imp"
        fi
    done <<< "$imports"
done

echo ""
if [[ $ERRORS -gt 0 ]]; then
    echo "FAILED: $ERRORS error(s) found."
    exit 1
else
    echo "All checks passed."
    exit 0
fi
