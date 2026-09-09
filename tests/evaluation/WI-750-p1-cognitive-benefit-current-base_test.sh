#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
python3 "$repo_root/tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py" \
  --repo "$repo_root" \
  --check
