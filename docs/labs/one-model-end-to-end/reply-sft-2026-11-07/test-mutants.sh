#!/usr/bin/env bash
# Prove the static gates catch changes to the registered stop and launch rules.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
JOB="$HERE/job.sh"
LAUNCHER="$HERE/run-s1.sh"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
caught=0
attempted=0

mutate() {
  local name=$1 file=$2 search=$3 replacement=$4
  attempted=$((attempted + 1))
  local target="$TMP/$name"
  cp "$file" "$target"
  SEARCH=$search REPLACE=$replacement perl -0pi -e \
    's/\Q$ENV{SEARCH}\E/$ENV{REPLACE}/g' "$target"
  cmp -s "$file" "$target" && {
    echo "mutation $name did not change its file" >&2
    exit 1
  }
  if "$HERE/gate.sh" \
    "$([ "$file" = "$JOB" ] && printf '%s' "$target" || printf '%s' "$JOB")" \
    "$([ "$file" = "$LAUNCHER" ] && printf '%s' "$target" || printf '%s' "$LAUNCHER")" \
    >/dev/null 2>&1; then
    echo "mutation $name escaped" >&2
    exit 1
  fi
  caught=$((caught + 1))
}

mutate wrong-stop-step "$JOB" '[ "$step" -eq 512 ]' '[ "$step" -eq 768 ]'
mutate weak-eos-p50 "$JOB" '[ "$eos_p50" -ge 7 ]' '[ "$eos_p50" -ge 0 ]'
mutate weak-guard "$JOB" '[ "$guard" -lt 93 ]' '[ "$guard" -lt 0 ]'
mutate no-freeze "$JOB" 'kill -STOP "$TRAIN_PID"' 'kill -CONT "$TRAIN_PID"'
mutate wrong-eval "$JOB" '--eval-every 256' '--eval-every 128'
mutate zero-replies "$JOB" '--max-replies 1 --reply-tokens 1' \
  '--max-replies 0 --reply-tokens 1'
mutate no-readback "$JOB" 'READBACK_STATUS=verified' 'READBACK_STATUS=unknown'
mutate two-gpus "$LAUNCHER" '--count 1 --hours 2' '--count 2 --hours 2'
mutate no-renew "$LAUNCHER" '"$POD_TOOL" renew "$POD"' 'true'

[ "$caught" -eq "$attempted" ]
echo "$caught of $attempted mutations caught"
