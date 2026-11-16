#!/usr/bin/env bash
# drive.sh STEP POD: the laptop side of claude-400 (D87).
#   note POD RATE_CENTS OFFER  record the lease (time now, $/h in cents, Vast offer id) in lease.env
#   put POD                    copy the instrument, pod.sh, bank and receipt inputs to the pod, and
#                              restore every finished file of $LOCAL/out (a re-leased pod resumes from
#                              them, never from an empty out/); refuses if the pod's out/ is not empty
#   machine POD                write machine.json from the lease and the pod's gpus.csv, and push it
#   sync POD [final]           copy finished shards/counts off the pod, checked by sha256; with
#                              "final", the logs are checked by sha256 too
#   wait-segment POD           block until the current run segment has stopped (tells the driver)
#   watch POD                  the guard (L-004), armed right after the lease and outliving segments:
#                              syncs every 10 min; stops the run and downs the pod at $1,100 or 60 h;
#                              downs the pod after a verified final sync once no run has been alive
#                              for GRACE s (30 min; 3 h before the first segment, for setup); downs
#                              it after 30 min unreachable. It exits only after `down` (or on a
#                              final-sync mismatch, with the pod kept so nothing unique is lost).
#   pod POD STEP...            run a pod.sh step on the pod
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
step=${1:?usage: drive.sh note|put|machine|sync|wait-segment|watch|pod POD ...}; POD=${2:?POD}; shift 2
ssh_pod() { "$UP" ssh "$POD" "$@"; }
log() { echo "$(date -u +%FT%TZ) $*"; }

# ALIVE, DEAD or UNREACH; an SSH failure is retried three times, 30 s apart, before UNREACH.
probe() {
  local s
  for _ in 1 2 3; do
    s=$(ssh_pod "if kill -0 \$(cat $W/run.pid 2>/dev/null) 2>/dev/null; then echo ALIVE; else echo DEAD; fi" 2>/dev/null | tail -1 || true)
    case $s in ALIVE|DEAD) echo "$s"; return;; esac
    sleep 30
  done
  echo UNREACH
}

do_sync() {
  local final=${1:-}
  mkdir -p "$LOCAL/out"
  local pat='shard-*.jsonl counts-*.jsonl'
  [ "$final" = final ] && pat="$pat rate.jsonl segments.jsonl dropped.jsonl metrics.json"
  # Final (renamed) files only; .part files are still being written.
  local list
  list=$(ssh_pod "cd $W/out 2>/dev/null && sha256sum $pat 2>/dev/null | grep -v '\.part' || true")
  local need=()
  while read -r sum name; do
    [ -n "$name" ] || continue
    if [ ! -f "$LOCAL/out/$name" ] || [ "$(shasum -a 256 "$LOCAL/out/$name" | cut -d' ' -f1)" != "$sum" ]; then need+=("$name"); fi
  done <<< "$list"
  if [ ${#need[@]} -gt 0 ]; then
    ssh_pod "cd $W/out && tar cf - ${need[*]}" | tar xf - -C "$LOCAL/out"
  fi
  # Between final syncs the logs are still growing: copied each time, checked only by a final sync.
  [ "$final" = final ] || { ssh_pod "cd $W/out && tar cf - rate.jsonl segments.jsonl dropped.jsonl metrics.json 2>/dev/null || true" | tar xf - -C "$LOCAL/out" 2>/dev/null || true; }
  local bad=0
  while read -r sum name; do
    [ -n "$name" ] || continue
    [ "$(shasum -a 256 "$LOCAL/out/$name" | cut -d' ' -f1)" = "$sum" ] || { log "MISMATCH $name"; bad=1; }
  done <<< "$list"
  log "synced${final:+ ($final)} $(echo "$list" | grep -c . || true) files (${#need[@]} new), mismatches=$bad; $(tail -1 "$LOCAL/out/rate.jsonl" 2>/dev/null || true)"
  return $bad
}

# A verified final sync (three tries), then down. On a persistent mismatch the pod is kept.
finish() {
  local why=$1 n
  for n in 1 2 3; do
    if do_sync final; then
      log "final sync verified ($why); down"
      "$UP" down "$POD" --lab claude --session claude-400 --confirm-archived && { log "pod down ($why)"; exit 0; }
      log "down failed (try $n)"
    fi
    sleep 60
  done
  log "FINISH FAILED ($why): pod $POD kept up; sync or down did not complete"; exit 1
}

case $step in
note)
  RATE=${1:?RATE_CENTS}; OFFER=${2:?OFFER}
  mkdir -p "$LOCAL"
  printf 'M2_POD=%s\nM2_LEASE_EPOCH=%s\nM2_RATE_CENTS=%s\nM2_OFFER=%s\n' "$POD" "$(date +%s)" "$RATE" "$OFFER" > "$LOCAL/lease.env"
  cat "$LOCAL/lease.env"
  ;;
put)
  if [ -n "$(ssh_pod "ls $W/out 2>/dev/null | head -1" || true)" ]; then echo "pod $W/out is not empty: refusing" >&2; exit 1; fi
  T=$(mktemp -d "${TMPDIR:-/tmp}/m2put.XXXX")
  mkdir -p "$T/instrument"
  cp -R "$HERE/instrument/Cargo.toml" "$HERE/instrument/Cargo.lock" "$HERE/instrument/src" "$T/instrument/"
  cp "$HERE/pod.sh" "$T/"
  for f in bank.jsonl bank.jsonl.manifest.json model.json machine.json engine.json; do
    [ -f "$LOCAL/$f" ] && cp "$LOCAL/$f" "$T/"
  done
  tar cf - -C "$T" . | ssh_pod "mkdir -p $W && tar xf - -C $W && chmod +x $W/pod.sh && ls $W"
  rm -rf "$T"
  # Restore finished work so a re-leased pod resumes instead of starting the bank again.
  if ls "$LOCAL/out"/shard-*.jsonl >/dev/null 2>&1; then
    (cd "$LOCAL/out" && ls shard-*.jsonl counts-*.jsonl dropped.jsonl rate.jsonl segments.jsonl metrics.json 2>/dev/null | grep -v '\.part' > "$LOCAL/restore.list")
    (cd "$LOCAL/out" && shasum -a 256 $(cat "$LOCAL/restore.list")) > "$LOCAL/restore.sha256"
    (cd "$LOCAL/out" && tar cf - $(cat "$LOCAL/restore.list")) | ssh_pod "mkdir -p $W/out && tar xf - -C $W/out"
    # Every restored file must read back "OK" on the pod; anything else stops before a run starts.
    got=$(ssh_pod "cd $W/out && sha256sum -c" < "$LOCAL/restore.sha256" 2>&1 || true)
    ok=$(echo "$got" | grep -c ': OK$' || true); want=$(wc -l < "$LOCAL/restore.list" | tr -d ' ')
    [ "$ok" = "$want" ] || { echo "$got" | grep -v ': OK$' >&2; echo "restore check FAILED: $ok of $want OK" >&2; exit 1; }
    log "restored $want files to $W/out, sha256 OK"
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
wait-segment)
  while :; do
    s=$(probe)
    [ "$s" = ALIVE ] || { log "segment ended ($s)"; do_sync || true; ssh_pod "tail -1 $W/out/segments.jsonl" || true; exit 0; }
    sleep 60
  done
  ;;
watch)
  # shellcheck disable=SC1091
  . "$LOCAL/lease.env"
  idle=0; unreach=0; seen=0
  while :; do
    now=$(date +%s); el=$((now - M2_LEASE_EPOCH)); spent=$((el * M2_RATE_CENTS / 3600))
    s=$(probe)
    log "state=$s elapsed=${el}s spent_cents=$spent idle=${idle}s"
    if [ "$spent" -ge "$CAP_CENTS" ] || [ "$el" -ge "$CAP_SECONDS" ]; then
      log "HARD STOP: spent_cents=$spent elapsed=${el}s"
      ssh_pod "touch $W/out/STOP" || true
      for _ in $(seq 1 30); do [ "$(probe)" = ALIVE ] || break; sleep 60; done
      finish "hard stop"
    fi
    case $s in
    ALIVE) seen=1; idle=0; unreach=0; do_sync || true ;;
    DEAD)
      unreach=0; do_sync || true
      g=$GRACE; [ "$seen" = 1 ] || g=$GRACE_FIRST
      [ "$idle" -ge "$g" ] && finish "no run for ${idle}s"
      idle=$((idle + EVERY)) ;;
    UNREACH)
      unreach=$((unreach + EVERY))
      if [ "$unreach" -ge 1800 ]; then
        log "pod unreachable for ${unreach}s: down without a final sync (the last verified sync holds the shards)"
        "$UP" down "$POD" --lab claude --session claude-400 --confirm-archived || log "DOWN FAILED: destroy $POD by hand"
        exit 3
      fi ;;
    esac
    sleep "$EVERY"
  done
  ;;
pod) ssh_pod "bash $W/pod.sh $*" ;;
*) echo "unknown step $step" >&2; exit 1 ;;
esac
