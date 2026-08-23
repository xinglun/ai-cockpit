#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd -P)"
PYTHONDONTWRITEBYTECODE=1 python3 "$repo_root/tests/ci/repository_gate_manifest_test.py"
