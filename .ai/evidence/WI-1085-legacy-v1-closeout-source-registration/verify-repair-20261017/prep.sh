#!/usr/bin/env bash
set -euo pipefail
for package in cockpit-agent cockpit-cli cockpit-core cockpit-evidence cockpit-git cockpit-interface cockpit-isolation cockpit-knowledge cockpit-mcp cockpit-protocol cockpit-release cockpit-repository cockpit-verification; do
    set +e
    cargo test --locked --package "$package" --no-run > "$WI1085_PREP_EVIDENCE/prep/$package.out.log" 2> "$WI1085_PREP_EVIDENCE/prep/$package.err.log"
    package_exit=$?
    set -e
    printf '%s\n' "$package_exit" > "$WI1085_PREP_EVIDENCE/prep/$package.exit.txt"
    printf '%s exit=%s\n' "$package" "$package_exit"
    if ((package_exit != 0)); then exit "$package_exit"; fi
done
