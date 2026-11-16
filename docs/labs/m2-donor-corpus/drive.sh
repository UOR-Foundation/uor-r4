#!/usr/bin/env bash
# drive.sh STEP POD: the laptop side of claude-400 (D87).
#   note POD RATE_CENTS OFFER  record a lease (time now, $/h in whole cents, Vast offer id) in lease.env;
#                              a re-lease carries the earlier pods' cents and the first lease time, so the
#                              guard's $1,100 and 60 h stops count every pod of the run
#   put POD                    copy the instrument, pod.sh, bank and receipt inputs to the pod, and restore
#                              every finished file of $LOCAL/out (a re-leased pod resumes from them); it
#                              refuses a pod whose out/ is not empty, and checks each restored file by sha256
#   machine POD                write machine.json from the lease and the pod's gpus.csv, and push it
#   sync POD [final]           copy finished shards/counts off the pod, checked by sha256 (a finished shard
#                              already on the laptop is never overwritten); "final" checks the logs too
#   hold POD                   tell the guard a vLLM restart is under way (counts as busy for 2 h)
#   wait-segment POD           block until the current run segment has stopped (tells the driver)
#   watch POD                  the guard (L-004), started detached right after the lease; see guard_help
#   pod POD STEP...            run a pod.sh step on the pod
guard_help='The guard syncs every 10 min. It stops the run and downs the pod at $1,100 or 60 h (all pods).
It downs the pod after a verified final sync once no run has been alive for 30 min (3 h before the
first segment). It downs it after 30 min unreachable. It never exits while the pod is up: a failed
sync or down is retried every 5 min with an alert (stdout, ALERTS file, desktop notification).'
set -euo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$HERE/../../.." && pwd)
UP=${UOR_POD:-$ROOT/scripts/pod/uor-pod}
LOCAL=${M2_LOCAL:-$ROOT/local/m2-run}
W=${M2_POD_W:-/workspace/m2}
CAP_CENTS=${M2_CAP_CENTS:-110000}
CAP_SECONDS=${M2_CAP_SECONDS:-216000}
GRACE=${M2_GRACE:-1800}
GRACE_FIRST=${M2_GRACE_FIRST:-10800}
EVERY=${M2_EVERY:-600}
RETRY=${M2_RETRY:-300}
UNREACH_MAX=${M2_UNREACH_MAX:-1800}
STOP_WAIT=${M2_STOP_WAIT:-1800}
KILL_WAIT=${M2_KILL_WAIT:-60}
POLL=${M2_POLL:-60}
LOGS='rate.jsonl segments.jsonl dropped.jsonl metrics.json'
step=${1:?usage: drive.sh note|put|machine|sync|hold|wait-segment|watch|pod POD ...}; POD=${2:?POD}; shift 2
ssh_pod() { "$UP" ssh "$POD" "$@"; }
log() { echo "$(date -u +%FT%TZ) $*"; }
alert() {
  log "ALERT: $*"
  mkdir -p "$LOCAL"; echo "$(date -u +%FT%TZ) $POD $*" >> "$LOCAL/ALERTS"
  command -v osascript >/dev/null && osascript -e "display notification \"$POD: $*\" with title \"claude-400 guard\"" >/dev/null 2>&1 || true
}

# ALIVE, HOLD, DEAD or UNREACH; an SSH failure is retried three times, 30 s apart, before UNREACH.
probe() {
  local s
  for _ in 1 2 3; do
    s=$(ssh_pod "if kill -0 \$(cat $W/run.pid 2>/dev/null) 2>/dev/null; then echo ALIVE; elif [ -n \"\$(find $W/HOLD -mmin -120 2>/dev/null)\" ]; then echo HOLD; else echo DEAD; fi" 2>/dev/null | tail -1 || true)
    case $s in ALIVE|HOLD|DEAD) echo "$s"; return;; esac
    sleep 30
  done
  echo UNREACH
}

# 0: synced and verified. 1: a mismatch. 2: no trustworthy listing (transport, or an empty final list
# after a run started). 3: a restore is in progress on the pod (nothing is copied).
do_sync() {
  local final=${1:-} pat='shard-*.jsonl counts-*.jsonl' raw list
  mkdir -p "$LOCAL/out"
  [ "$final" = final ] && pat="$pat $LOGS"
  # The remote prints its flags, the sums of final files only (.part files are still being written),
  # then END; a listing without END is not trusted.
  raw=$(ssh_pod "cd $W/out 2>/dev/null || { echo NOOUT; echo END; exit 0; }; [ -e .restoring ] && echo RESTORING; [ -e $W/run.pid ] && echo RAN; sha256sum $pat 2>/dev/null | grep -v '\.part' || true; echo END" 2>/dev/null) || { log "sync: listing failed (ssh)"; return 2; }
  [ "$(echo "$raw" | tail -1)" = END ] || { log "sync: listing has no END"; return 2; }
  echo "$raw" | grep -qx RESTORING && { log "sync: restore in progress, skipped"; return 3; }
  list=$(echo "$raw" | grep -v -x -E 'END|NOOUT|RAN|RESTORING' || true)
  if [ "$final" = final ] && [ -z "$list" ] && echo "$raw" | grep -qx RAN; then log "sync: empty final listing after a run started"; return 2; fi
  local need=() bad=0 sum name
  while read -r sum name; do
    [ -n "$name" ] || continue
    case $name in
    shard-*|counts-*)
      # Finished shards are immutable: copy a missing one; a differing local copy is a mismatch, never overwritten.
      if [ ! -f "$LOCAL/out/$name" ]; then need+=("$name")
      elif [ "$(shasum -a 256 "$LOCAL/out/$name" | cut -d' ' -f1)" != "$sum" ]; then log "MISMATCH (kept local) $name"; bad=1; fi ;;
    *) need+=("$name") ;;
    esac
  done <<< "$list"
  if [ ${#need[@]} -gt 0 ]; then
    ssh_pod "cd $W/out && tar cf - ${need[*]}" | tar xf - -C "$LOCAL/out" || { log "sync: copy failed"; return 2; }
  fi
  # Between final syncs the logs are still growing: copied each time, checked only by a final sync.
  [ "$final" = final ] || { ssh_pod "cd $W/out && tar cf - $LOGS 2>/dev/null || true" | tar xf - -C "$LOCAL/out" 2>/dev/null || true; }
  for name in "${need[@]+"${need[@]}"}"; do
    sum=$(echo "$list" | awk -v n="$name" '$2 == n {print $1}')
    [ "$(shasum -a 256 "$LOCAL/out/$name" | cut -d' ' -f1)" = "$sum" ] || { log "MISMATCH $name"; bad=1; }
  done
  log "synced${final:+ ($final)} $(echo "$list" | grep -c . || true) files (${#need[@]} copied), mismatches=$bad; $(tail -1 "$LOCAL/out/rate.jsonl" 2>/dev/null || true)"
  return $bad
}

pod_down() {
  "$UP" down "$POD" --lab claude --session claude-400 --confirm-archived || return 1
  echo "M2_DOWN_EPOCH=$(date +%s)" >> "$LOCAL/lease.env"
}

# Stop the run if it is still alive: STOP, then TERM and KILL after STOP_WAIT.
stop_run() {
  ssh_pod "touch $W/out/STOP" || true
  local t=0
  while [ "$(probe)" = ALIVE ] && [ "$t" -lt "$STOP_WAIT" ]; do sleep "$POLL"; t=$((t + POLL)); done
  if [ "$(probe)" = ALIVE ]; then
    alert "run still alive ${t}s after STOP: TERM, then KILL"
    ssh_pod "kill -TERM \$(cat $W/run.pid); sleep $KILL_WAIT; kill -KILL \$(cat $W/run.pid) 2>/dev/null; true" || true
  fi
}

# A verified final sync, then down. It never returns while the pod is up.
finish() {
  local why=$1 rc unreach=0
  while :; do
    rc=0; do_sync final || rc=$?
    if [ "$rc" = 0 ]; then
      log "final sync verified ($why); down"
      if pod_down; then log "pod down ($why)"; exit 0; fi
      alert "down failed after a verified sync ($why); retrying every ${RETRY}s"
    elif [ "$rc" = 2 ] && [ "$(probe)" = UNREACH ]; then
      unreach=$((unreach + RETRY))
      if [ "$unreach" -ge "$UNREACH_MAX" ]; then
        alert "unreachable ${unreach}s during finish: down without a final sync (the last verified sync holds the shards)"
        if pod_down; then exit 3; fi
      fi
    else
      alert "final sync not verified (code $rc, $why); pod kept up; retrying every ${RETRY}s"
    fi
    sleep "$RETRY"
  done
}

case $step in
note)
  RATE=${1:?RATE_CENTS}; OFFER=${2:?OFFER}
  [[ $RATE =~ ^[0-9]{3,5}$ ]] || { echo "RATE_CENTS must be whole cents per hour, e.g. 2308" >&2; exit 1; }
  mkdir -p "$LOCAL"
  now=$(date +%s); prior=0; first=$now
  if [ -f "$LOCAL/lease.env" ]; then
    # shellcheck disable=SC1091
    . "$LOCAL/lease.env"
    end=${M2_DOWN_EPOCH:-$now}
    prior=$(( ${M2_PRIOR_CENTS:-0} + (end - M2_LEASE_EPOCH) * M2_RATE_CENTS / 3600 ))
    first=${M2_FIRST_LEASE_EPOCH:-$M2_LEASE_EPOCH}
    mv "$LOCAL/lease.env" "$LOCAL/lease-$M2_POD.env"
  fi
  printf 'M2_POD=%s\nM2_LEASE_EPOCH=%s\nM2_RATE_CENTS=%s\nM2_OFFER=%s\nM2_PRIOR_CENTS=%s\nM2_FIRST_LEASE_EPOCH=%s\n' \
    "$POD" "$now" "$RATE" "$OFFER" "$prior" "$first" > "$LOCAL/lease.env"
  cat "$LOCAL/lease.env"
  ;;
put)
  if [ -n "$(ssh_pod "ls -A $W/out 2>/dev/null | head -1")" ]; then echo "pod $W/out is not empty: refusing" >&2; exit 1; fi
  T=$(mktemp -d "${TMPDIR:-/tmp}/m2put.XXXX")
  mkdir -p "$T/instrument"
  cp -R "$HERE/instrument/Cargo.toml" "$HERE/instrument/Cargo.lock" "$HERE/instrument/src" "$T/instrument/"
  cp "$HERE/pod.sh" "$T/"
  for f in bank.jsonl bank.jsonl.manifest.json model.json machine.json engine.json; do
    if [ -f "$LOCAL/$f" ]; then cp "$LOCAL/$f" "$T/"; fi
  done
  tar cf - -C "$T" . | ssh_pod "mkdir -p $W && tar xf - -C $W && chmod +x $W/pod.sh && ls $W"
  rm -rf "$T"
  # Restore finished work so a re-leased pod resumes instead of starting the bank again. The pod's
  # .restoring marker keeps the guard from syncing a half-extracted file meanwhile.
  files=()
  for f in "$LOCAL"/out/shard-*.jsonl "$LOCAL"/out/counts-*.jsonl; do [ -f "$f" ] && files+=("$(basename "$f")"); done
  for f in $LOGS; do [ -f "$LOCAL/out/$f" ] && files+=("$f"); done
  if [ ${#files[@]} -gt 0 ]; then
    (cd "$LOCAL/out" && shasum -a 256 "${files[@]}") > "$LOCAL/restore.sha256"
    ssh_pod "mkdir -p $W/out && touch $W/out/.restoring"
    (cd "$LOCAL/out" && tar cf - "${files[@]}") | ssh_pod "tar xf - -C $W/out"
    # Every restored file must read back "OK" on the pod; anything else stops before a run starts.
    got=$(ssh_pod "cd $W/out && sha256sum -c -" < "$LOCAL/restore.sha256" 2>&1 || true)
    ok=$(echo "$got" | grep -c ': OK$' || true)
    [ "$ok" = "${#files[@]}" ] || { echo "$got" | grep -v ': OK$' >&2; echo "restore check FAILED: $ok of ${#files[@]} OK" >&2; exit 1; }
    ssh_pod "rm -f $W/out/.restoring"
    log "restored ${#files[@]} files to $W/out, sha256 OK"
  fi
  ;;
machine)
  # shellcheck disable=SC1091
  . "$LOCAL/lease.env"
  gpus=$(ssh_pod "cat $W/gpus.csv")
  mem=$(ssh_pod "cat $W/host-mem.txt")
  vv=$(ssh_pod "cat $W/vllm-version.txt")
  jq -n --arg pod "$M2_POD" --arg offer "$M2_OFFER" --argjson cents "$M2_RATE_CENTS" \
    --argjson lease "$M2_LEASE_EPOCH" --arg gpus "$gpus" --arg mem "$mem" --arg vv "$vv" \
    '{provider: "vast", pod: $pod, offer: $offer, rate_cents_per_hour: $cents, lease_epoch: $lease,
      gpus_csv: ($gpus | split("\n")), host_mem: ($mem | split("\n")), vllm_version: $vv}' > "$LOCAL/machine.json"
  ssh_pod "cat > $W/machine.json" < "$LOCAL/machine.json"
  jq -c '{pod, offer, gpus: (.gpus_csv | length)}' "$LOCAL/machine.json"
  ;;
sync) do_sync "${1:-}" ;;
hold) ssh_pod "touch $W/HOLD" && log "hold set: the guard counts the pod busy for 2 h (remove $W/HOLD to end it)" ;;
wait-segment)
  while :; do
    s=$(probe)
    [ "$s" = ALIVE ] || { log "segment ended ($s)"; do_sync || true; ssh_pod "tail -1 $W/out/segments.jsonl" || true; exit 0; }
    sleep 60
  done
  ;;
watch)
  echo "$guard_help"
  # shellcheck disable=SC1091
  . "$LOCAL/lease.env"
  first=${M2_FIRST_LEASE_EPOCH:-$M2_LEASE_EPOCH}; prior=${M2_PRIOR_CENTS:-0}
  seenf="$LOCAL/guard-seen-$POD"; seen=0; [ -f "$seenf" ] && seen=1
  idle=0; unreach=0
  while :; do
    now=$(date +%s); el=$((now - first)); spent=$((prior + (now - M2_LEASE_EPOCH) * M2_RATE_CENTS / 3600))
    s=$(probe)
    log "state=$s elapsed=${el}s spent_cents=$spent idle=${idle}s"
    if [ "$spent" -ge "$CAP_CENTS" ] || [ "$el" -ge "$CAP_SECONDS" ]; then
      alert "HARD STOP: spent_cents=$spent elapsed=${el}s"
      stop_run
      finish "hard stop"
    fi
    case $s in
    ALIVE) seen=1; touch "$seenf"; idle=0; unreach=0; do_sync || true ;;
    HOLD) idle=0; unreach=0; do_sync || true ;;
    DEAD)
      unreach=0; do_sync || true
      g=$GRACE; [ "$seen" = 1 ] || g=$GRACE_FIRST
      [ "$idle" -ge "$g" ] && finish "no run for ${idle}s"
      idle=$((idle + EVERY)) ;;
    UNREACH)
      unreach=$((unreach + EVERY))
      if [ "$unreach" -ge "$UNREACH_MAX" ]; then
        alert "unreachable ${unreach}s: down without a final sync (the last verified sync holds the shards)"
        if pod_down; then exit 3; fi
        alert "down failed while unreachable; retrying"
      fi ;;
    esac
    sleep "$EVERY"
  done
  ;;
pod) ssh_pod "bash $W/pod.sh $*" ;;
*) echo "unknown step $step" >&2; exit 1 ;;
esac
