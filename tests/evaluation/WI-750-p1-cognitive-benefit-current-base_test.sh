#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cargo run --locked --quiet --manifest-path "$repo_root/Cargo.toml" \
  -p cockpit-cli --bin ai-cockpit -- \
  audit cognitive-benefit --repo "$repo_root" --check
