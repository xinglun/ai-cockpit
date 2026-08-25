#!/usr/bin/env bash
set -euo pipefail

# Keep the corrective Work Item's Runtime evidence on one bounded command
# while preserving each underlying regression's own deterministic output.
bash tests/ci/governance_integrity_gate_test.sh
bash tests/docs/promote_closed_work_item_test.sh
bash tests/docs/documentation_acceptance.sh
