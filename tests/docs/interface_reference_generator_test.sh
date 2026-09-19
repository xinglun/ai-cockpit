#!/usr/bin/env bash
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
python3 "$repo/scripts/generate_interface_references.py" --repo "$repo" --check
