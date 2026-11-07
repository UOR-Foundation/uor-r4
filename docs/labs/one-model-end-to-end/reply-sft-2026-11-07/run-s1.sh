#!/usr/bin/env bash
# Launch the reviewed seed-1 reply-SFT job on one DeepSeek-owned RTX 5090.
#
# UOR_SFT_PIN must be the exact merge commit that contains this script and the
# reviewed trainer/diagnostic dependencies.
set -euo pipefail

PIN=${UOR_SFT_PIN:?UOR_SFT_PIN must name the reviewed 40-hex merge commit}
case "$PIN" in
  *[!0-9a-f]*|'') echo "UOR_SFT_PIN is not lowercase 40-hex" >&2; exit 2 ;;
esac
[ "${#PIN}" -eq 40 ] || { echo "UOR_SFT_PIN is not 40 hex characters" >&2; exit 2; }

ROOT=$(git rev-parse --show-toplevel)
POD_TOOL="$ROOT/scripts/pod/uor-pod"
STORE="$ROOT/scripts/pod/uor-store"
JOB="$ROOT/docs/labs/one-model-end-to-end/reply-sft-2026-11-07/job.sh"
RUN_ID=$(date -u +%Y%m%dT%H%M%SZ | tr '[:upper:]' '[:lower:]')-$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')
SESSION="deepseek-sft-$RUN_ID"
RUN="/workspace/uor-r4/runs/deepseek-sft-s1-$RUN_ID"
OUT_STORE="results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1-$RUN_ID"
STORE_TOPIC="results/deepseek/one-model-end-to-end-reply-sft-2026-11-07"
CONTROLLER="$ROOT/local/deepseek/one-model-sft-2026-11-07/$RUN_ID"
MAXSEC=${UOR_SFT_MAXSEC:-7200}
STALL_SEC=${UOR_SFT_STALL_SEC:-1800}
MODEL_TIME="$ROOT/.uor-models/native-joint-learning-2026-09-04/model-time.json"
STORAGE_RECEIPT=${UOR_STORAGE_RECEIPT:-"$ROOT/.uor-models/native-joint-learning-2026-09-04/storage-receipt.json"}
MIN_FREE_KIB=31588352

case "$MAXSEC" in ''|*[!0-9]*) echo "UOR_SFT_MAXSEC must be an integer" >&2; exit 2 ;; esac
[ "$MAXSEC" -le 21600 ] || { echo "UOR_SFT_MAXSEC may not exceed six hours" >&2; exit 2; }
case "$STALL_SEC" in ''|*[!0-9]*) echo "UOR_SFT_STALL_SEC must be an integer" >&2; exit 2 ;; esac
[ "$STALL_SEC" -ge 300 ] && [ "$STALL_SEC" -le 7200 ] ||
  { echo "UOR_SFT_STALL_SEC must be 300 to 7200 seconds" >&2; exit 2; }
[ -x "$POD_TOOL" ] || { echo "missing $POD_TOOL" >&2; exit 2; }
[ -x "$STORE" ] || { echo "missing $STORE" >&2; exit 2; }
[ -f "$JOB" ] || { echo "missing $JOB" >&2; exit 2; }
git cat-file -e "$PIN^{commit}" || { echo "local repository does not contain $PIN" >&2; exit 2; }
[ "$(git rev-parse HEAD)" = "$PIN" ] ||
  { echo "local HEAD is not the reviewed pin $PIN" >&2; exit 2; }
[ -z "$(git status --porcelain --untracked-files=no)" ] ||
  { echo "tracked files differ from the reviewed pin $PIN" >&2; exit 2; }
git fetch --quiet origin main || { echo "could not refresh origin/main" >&2; exit 2; }
git merge-base --is-ancestor "$PIN" origin/main ||
  { echo "$PIN is not merged on origin/main" >&2; exit 2; }

install -d "$CONTROLLER"
free_kib=$(df -Pk "$ROOT" | awk 'NR == 2 { print $4 }')
case "$free_kib" in ''|*[!0-9]*) echo "could not measure free space" >&2; exit 2 ;; esac
[ "$free_kib" -ge "$MIN_FREE_KIB" ] ||
  { echo "free space ${free_kib} KiB is below the 30 GiB + 128 MiB floor" >&2; exit 2; }
model_time_state=absent
model_time_value=null
if [ -f "$MODEL_TIME" ]; then
  jq empty "$MODEL_TIME" || { echo "model-time ledger is not valid JSON" >&2; exit 2; }
  model_time_state=present
  model_time_value=$(jq -c . "$MODEL_TIME")
fi
storage_receipt_state=absent
storage_receipt_value=null
if [ -f "$STORAGE_RECEIPT" ]; then
  jq empty "$STORAGE_RECEIPT" || { echo "storage receipt is not valid JSON" >&2; exit 2; }
  storage_receipt_state=present
  storage_receipt_value=$(jq -c . "$STORAGE_RECEIPT")
fi
cpu_count=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 0)
mem_available_kib=$(awk '/^MemAvailable:/ { print $2 }' /proc/meminfo 2>/dev/null || echo 0)
case "$cpu_count" in ''|*[!0-9]*) cpu_count=0 ;; esac
case "$mem_available_kib" in ''|*[!0-9]*) mem_available_kib=0 ;; esac
df -Pk "$ROOT" > "$CONTROLLER/df-before.txt"
free -b > "$CONTROLLER/free-before.txt"
jq -n --arg model_time_state "$model_time_state" --argjson model_time "$model_time_value" \
  --arg storage_state "$storage_receipt_state" --argjson storage "$storage_receipt_value" \
  --argjson free_kib "$free_kib" --argjson cpu_count "$cpu_count" \
  --argjson mem_available_kib "$mem_available_kib" \
  '{model_time_state:$model_time_state,model_time:$model_time,
    storage_receipt_state:$storage_state,storage_receipt:$storage,
    free_kib:$free_kib,floor_kib:31588352,cpu_count:$cpu_count,
    mem_available_kib:$mem_available_kib}' > "$CONTROLLER/resource-state.json"
JOB_SHA256=$(sha256sum "$JOB" | cut -d' ' -f1)
printf '%s  %s\n' "$JOB_SHA256" "$JOB" > "$CONTROLLER/job.sha256"
{
  printf 'export UOR_SFT_PIN=%q\n' "$PIN"
  printf 'export UOR_SFT_RUN=%q\n' "$RUN"
  printf 'export UOR_SFT_OUT_STORE=%q\n' "$OUT_STORE"
  printf 'export UOR_SFT_SESSION=%q\n' "$SESSION"
} > "$CONTROLLER/controller.env"

if ! listings=$("$STORE" ls "$STORE_TOPIC"); then
  echo "cannot verify the store topic $STORE_TOPIC; no pod will be rented" >&2
  exit 1
fi
if printf '%s\n' "$listings" |
  awk -v path="$OUT_STORE/" 'index($0, path) { found = 1 } END { exit(found ? 0 : 1) }'; then
  echo "refusing nonempty store root $OUT_STORE" >&2
  exit 1
fi

"$POD_TOOL" status | tee "$CONTROLLER/status-before.txt"
export UOR_POD_MAX_PODS=4
export UOR_POD_MAX_RATE=8.00

POD=
RESOLVED=0
STATUS=FAILED
JOB_LOG=
JOB_SCRIPT=
stop_remote_job() {
  [ -n "$POD" ] && [ -n "$JOB_SCRIPT" ] || return 0
  "$POD_TOOL" ssh "$POD" \
    "line=\$(ps -eo pid=,pgid=,args= |
       awk -v script='$JOB_SCRIPT' '
         /^[[:space:]]*[0-9]+[[:space:]]+[0-9]+[[:space:]]/ {
           for (i = 3; i <= NF; i++) if (\$i == script) { print \$1, \$2; exit }
         }')
     set -- \$line
     if [ \$# -eq 2 ]; then kill -TERM -- -\$2 2>/dev/null || true; fi" ||
    true
  local waited=0
  while [ "$waited" -lt 12 ]; do
    if "$POD_TOOL" ssh "$POD" 'flock -n /root/gpu0.lock true' >/dev/null 2>&1; then
      return 0
    fi
    sleep 5
    waited=$((waited + 1))
  done
  echo "remote SFT job still holds GPU 0 after termination; the pod is retained" >&2
  return 1
}
cleanup() {
  local rc=$?
  trap - EXIT INT TERM
  printf 'status=%s\n' "$STATUS" > "$CONTROLLER/status.txt"
  if [ -n "$POD" ] && [ "$RESOLVED" -ne 1 ]; then
    RESOLVED=1
    if [ "$STATUS" = SEALED ]; then
      "$POD_TOOL" release "$POD" --lab deepseek --session "$SESSION" || rc=1
      "$POD_TOOL" down "$POD" --lab deepseek --session "$SESSION" \
        --confirm-archived || rc=1
    else
      echo "run did not prove a sealed readback; archiving the pod before teardown" >&2
      stop_remote_job || rc=1
      "$POD_TOOL" down "$POD" --lab deepseek --session "$SESSION" --to-store || rc=1
    fi
  fi
  exit "$rc"
}
trap cleanup EXIT INT TERM

created=$("$POD_TOOL" up --lab deepseek --session "$SESSION" --gpu 5090 \
  --count 1 --hours 2 --ref "$PIN" \
  --purpose "reply-masked SFT from b64L seed 1" 2>&1 |
  tee "$CONTROLLER/up.log")
POD=$(printf '%s\n' "$created" |
  awk '$1 == "Pod" && $3 == "created;" { print $2; exit }')
[ -n "$POD" ] || { echo "could not parse the pod id" >&2; exit 1; }
printf '%s\n' "$POD" > "$CONTROLLER/pod"

"$POD_TOOL" ssh "$POD" 'cat > /workspace/one-model-reply-sft-job.sh' < "$JOB"
"$POD_TOOL" ssh "$POD" \
  "printf '%s  /workspace/one-model-reply-sft-job.sh\n' '$JOB_SHA256' | sha256sum -c -" |
  tee "$CONTROLLER/job-upload.sha256"

started=$(date +%s)
if ! "$POD_TOOL" run "$POD" --lab deepseek --session "$SESSION" --gpu 0 -- \
  env UOR_SFT_PIN="$PIN" UOR_SFT_RUN_ID="$RUN_ID" UOR_SFT_RUN="$RUN" \
  UOR_SFT_OUT_STORE="$OUT_STORE" \
  bash /workspace/one-model-reply-sft-job.sh 1 \
  > "$CONTROLLER/run.out" 2>&1; then
  cat "$CONTROLLER/run.out" >&2
  echo "could not submit the remote SFT job" >&2
  exit 1
fi
JOB_LOG=$(awk '/Log: / {
  for (i = 1; i <= NF; i++) if ($i == "Log:") { print $(i + 1); exit }
}' "$CONTROLLER/run.out")
case "$JOB_LOG" in
  /workspace/uor-r4/jobs/*) ;;
  *) cat "$CONTROLLER/run.out" >&2; echo "uor-pod did not report a job log" >&2; exit 1 ;;
esac
JOB_SCRIPT="${JOB_LOG%.log}.sh"
case "$JOB_SCRIPT" in
  /workspace/uor-r4/jobs/*.sh) ;;
  *) echo "uor-pod reported an invalid job script path" >&2; exit 1 ;;
esac
printf '%s\n' "$JOB_LOG" > "$CONTROLLER/job-log.path"
last_renew=$started
failures=0

while :; do
  probe=$("$POD_TOOL" ssh "$POD" \
    "exit=\$(sed -n 's/^# exit=\\([0-9][0-9]*\\) .*/\\1/p' '$JOB_LOG' 2>/dev/null | tail -1)
     if [ -n \"\$exit\" ]; then printf 'exit %s\\n' \"\$exit\"
     elif ! flock -n /root/gpu0.lock true; then printf 'running\\n'
     else printf 'stopped\\n'; fi") || probe=unreachable
  case "$probe" in
    'exit '*) job_exit=${probe#exit }; break ;;
    running) failures=0 ;;
    *) failures=$((failures + 1)) ;;
  esac
  now=$(date +%s)
  progress_line=$("$POD_TOOL" ssh "$POD" \
    "now=\$(date +%s)
     latest=\$(find '$RUN' -type f -printf '%T@\\n' 2>/dev/null | sort -nr | head -1 | cut -d. -f1)
     printf '%s %s\\n' \"\$now\" \"\${latest:-0}\"") || progress_line=
  read -r remote_now latest_progress <<< "$progress_line" || true
  case "$remote_now:$latest_progress" in
    *[!0-9:]*|:*) ;;
    *)
      if [ $((remote_now - latest_progress)) -ge "$STALL_SEC" ]; then
        echo "$POD has made no file progress for at least $STALL_SEC seconds" >&2
        exit 1
      fi
      ;;
  esac
  if [ $((now - started)) -gt "$MAXSEC" ]; then
    echo "watcher time limit reached; the healthy job is not cancelled" |
      tee -a "$CONTROLLER/watch.log"
    started=$now
  fi
  if [ $((now - last_renew)) -ge 1500 ]; then
    if ! "$POD_TOOL" renew "$POD" --lab deepseek --session "$SESSION" --hours 2 \
      >> "$CONTROLLER/watch.log" 2>&1; then
      echo "could not renew $POD" >&2
      exit 1
    fi
    last_renew=$now
  fi
  if [ "$failures" -ge 3 ]; then
    if ! "$POD_TOOL" status 2>/dev/null | grep -Eq "^$POD .* RUNNING "; then
      echo "$POD disappeared before writing its terminal exit" >&2
      exit 1
    fi
    echo "$POD is running, but GPU 0 has no live job and $JOB_LOG has no terminal exit" >&2
    exit 1
  fi
  sleep 20
done

"$POD_TOOL" ssh "$POD" "tail -n 200 '$JOB_LOG'" |
  tee "$CONTROLLER/job-tail.log"
[ "$job_exit" -eq 0 ] || {
  echo "remote SFT job exited $job_exit; the pod is retained for its failure seal" >&2
  exit 1
}
mkdir -p "$CONTROLLER/readback"
"$STORE" get "$OUT_STORE/READBACK" "$CONTROLLER/readback" >/dev/null
READBACK_FILE="$CONTROLLER/readback/$OUT_STORE/READBACK"
[ -f "$READBACK_FILE" ] || { echo "no READBACK file was returned" >&2; exit 1; }
[ "$(cat "$READBACK_FILE")" = verified ] || {
  echo "store readback is $(cat "$READBACK_FILE"), not verified" >&2
  exit 1
}
"$STORE" get "$OUT_STORE/EXIT" "$CONTROLLER/readback" >/dev/null
[ "$(cat "$CONTROLLER/readback/$OUT_STORE/EXIT")" = 0 ] || {
  echo "sealed EXIT is not zero" >&2
  exit 1
}
STATUS=SEALED
echo "sealed and read back $OUT_STORE"
