#!/bin/sh
# >>> uor-harness pre-push v1 (managed by crates/uor-r4-harness) >>>
# Installed by: uor-harness git install-hooks
# Remove with:  rm "$(git rev-parse --git-path hooks)/pre-push"
#
# Runs uor-harness git push-check on the refs git passes on stdin: for every
# pushed <lab>/... branch it runs loop pr-check --local on the pushed head, and
# it skips other refs with a printed reason.
# It exits 0 when uor-harness is not on PATH, and when the harness itself cannot
# produce a result (any exit other than 0 or 1).
set -u
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -n "$root" ] || exit 0
command -v uor-harness >/dev/null 2>&1 || exit 0
uor-harness git push-check --repo "$root"
rc=$?
if [ "$rc" -eq 1 ]; then
    echo "uor-harness: push blocked. Bypass (not recommended): git push --no-verify"
    exit 1
fi
exit 0
# <<< uor-harness pre-push v1 <<<
