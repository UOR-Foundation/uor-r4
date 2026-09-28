#!/usr/bin/env bash
# E2E Test Suite Runner for UOR-R4 Geometric Language Model
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo "=== UOR-R4 Geometric Language Model E2E Test Suite ==="
echo "Worktree : ${WORKTREE_DIR}"
echo "Script   : ${SCRIPT_DIR}/run_e2e_tests.py"
echo "Starting test execution across Tiers 1-4..."
echo

python3 "${SCRIPT_DIR}/run_e2e_tests.py" "$@"
