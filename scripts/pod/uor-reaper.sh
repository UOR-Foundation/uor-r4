#!/usr/bin/env bash
# Pod-side reaper for the shared UOR-R4 GPU pods (#820; docs/labs/compute.md).
# Installed at /root/uor-reaper.sh and started (nohup) by uor-pod-bootstrap.sh
# only on pods created by `uor-pod up`.
#
#   uor-reaper.sh --loop    every 60 s: when no /root/leases/*.json is unexpired
#                           AND the GPUs have been idle >= UOR_REAPER_IDLE_MIN
#                           (default 20) minutes, delete this pod through the
#                           Runpod API with the pod's own RUNPOD_API_KEY
#   uor-reaper.sh --check   verify that key can read this pod (prints api-ok/FAILED)
#
# Busy = any GPU above 2 % utilisation, any CUDA compute process, or a held
# /root/gpuK.lock (`uor-pod run` jobs, including queued ones).
# The key is read from /proc/1/environ at the moment of use and passed to curl
# on stdin, never in argv, a file or the log. Log: /workspace/uor-r4/pods/reaper.log
set -uo pipefail

IDLE_MIN=${UOR_REAPER_IDLE_MIN:-20}
INTERVAL=${UOR_REAPER_INTERVAL:-60}
LOG=/workspace/uor-r4/pods/reaper.log
SINCE=/root/.uor-idle-since
env1() { tr '\0' '\n' < /proc/1/environ | sed -n "s/^$1=//p" | head -1; }
POD=$(env1 RUNPOD_POD_ID)
mkdir -p "$(dirname "$LOG")"
log() { echo "$(date -u +%FT%TZ) pod=$POD $*" >> "$LOG"; }

api() {  # api METHOD URL -> HTTP status; the Authorization header comes from stdin
  printf 'Authorization: Bearer %s\n' "$(env1 RUNPOD_API_KEY)" |
    curl -sS -o /dev/null -w '%{http_code}' -X "$1" -H @- --max-time 30 "$2"
}

live_lease() {
  python3 - <<'PY'
import glob, json, sys, time, calendar
for f in glob.glob('/root/leases/*.json'):
    try:
        exp = json.load(open(f))['expires']
        if calendar.timegm(time.strptime(exp, '%Y-%m-%dT%H:%M:%SZ')) > time.time():
            sys.exit(0)
    except Exception:
        sys.exit(0)  # unreadable lease: treat as live (never delete on doubt)
sys.exit(1)
PY
}

busy() {
  local f
  nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits 2>/dev/null |
    awk '$1 > 2 { b = 1 } END { exit !b }' && return 0
  [ -n "$(nvidia-smi --query-compute-apps=pid --format=csv,noheader 2>/dev/null)" ] && return 0
  for f in /root/gpu*.lock; do
    [ -e "$f" ] || continue
    flock -n "$f" true || return 0
  done
  return 1
}

case ${1:-} in
  --check)
    [ -n "$POD" ] || { echo "api-FAILED (no RUNPOD_POD_ID)"; exit 1; }
    [ -n "$(env1 RUNPOD_API_KEY)" ] || { echo "api-FAILED (no RUNPOD_API_KEY in the pod environment)"; exit 1; }
    code=$(api GET "https://rest.runpod.io/v1/pods/$POD")
    if [ "$code" = 200 ]; then echo api-ok; exit 0; fi
    echo "api-FAILED (HTTP $code)"; exit 1;;
  --loop) ;;
  *) sed -n '2,17p' "$0"; exit 2;;
esac

log "reaper start (idle limit $IDLE_MIN min)"
date +%s > "$SINCE"   # grace: a fresh pod or restarted reaper waits a full period
while :; do
  if live_lease || busy; then
    date +%s > "$SINCE"
  else
    idle=$(( $(date +%s) - $(cat "$SINCE" 2>/dev/null || date +%s) ))
    if [ "$idle" -ge $(( IDLE_MIN * 60 )) ]; then
      log "no live lease and GPUs idle $(( idle / 60 )) min: deleting this pod"
      code=$(api DELETE "https://rest.runpod.io/v1/pods/$POD")
      log "delete request HTTP $code"
      case $code in 2*) sleep 600;; *) sleep 120;; esac
    fi
  fi
  sleep "$INTERVAL"
done
