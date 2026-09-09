#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
python3 "$repo_root/tests/evaluation/WI-745-p1-cognitive-benefit-evaluation.py" \
  --repo "$repo_root" \
  --check
