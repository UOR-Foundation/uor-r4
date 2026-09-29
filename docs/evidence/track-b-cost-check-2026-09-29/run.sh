#!/bin/bash
set -uo pipefail
cd /Users/casey.allard/.codex/worktrees/track-b-conversion/uor-r4 || exit 1
out=/tmp/codex-track-b-cost-check-20260929-1
mkdir "$out" || exit 1
ulimit -f 32768
started=$(date -u +%FT%TZ)
source_sha=$(shasum -a 256 crates/uor-r4-training/drafts/track_b_cost.rs | awk '{print $1}')
child=''
watcher=''
cleanup() {
  if [ -n "$watcher" ]; then
    pkill -TERM -P "$watcher" 2>/dev/null || true
    kill -TERM "$watcher" 2>/dev/null || true
    wait "$watcher" 2>/dev/null || true
  fi
}
trap cleanup EXIT
/usr/bin/time -l /Users/casey.allard/.cargo/bin/rustc --edition=2021 --test -C codegen-units=1 crates/uor-r4-training/drafts/track_b_cost.rs -o "$out/cost-tests" > "$out/compile.log" 2>&1 &
child=$!
(sleep 60 || exit 0; pkill -TERM -P "$child" 2>/dev/null || true; kill -TERM "$child" 2>/dev/null || true) &
watcher=$!
wait "$child"
compile_exit=$?
cleanup
watcher=''
test_exit=125
if [ "$compile_exit" -eq 0 ]; then
  /usr/bin/time -l "$out/cost-tests" --test-threads=1 > "$out/tests.log" 2>&1
  test_exit=$?
fi
jq -n --arg started "$started" --arg ended "$(date -u +%FT%TZ)" --arg source_sha "$source_sha" --argjson compile_exit "$compile_exit" --argjson test_exit "$test_exit" '{schema:"uor-r4.track-b-analytic-check/1",status:"EXECUTED_ACCOUNTING_FIXTURES_ONLY",started_utc:$started,ended_utc:$ended,source_sha256:$source_sha,compile_exit:$compile_exit,test_exit:$test_exit,model_work:"NOT_RUN",cargo_used:false}' > "$out/receipt.json"
cat "$out/compile.log" "$out/tests.log" "$out/receipt.json"
[ "$compile_exit" -eq 0 ] && [ "$test_exit" -eq 0 ]
