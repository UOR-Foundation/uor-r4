#!/usr/bin/env bash
# drive.sh STEP POD: the laptop side of claude-400 (D87).
#   put POD          copy the instrument source, pod.sh, bank and receipt inputs to the pod
#   sync POD         copy finished shards/counts off the pod once, checked by sha256
#   watch POD        sync every 10 min until the run has stopped and everything is copied;
#                    the completion watcher (L-004): it exits when there is nothing left to do
#   pod POD STEP...  run a pod.sh step on the pod
set -euo pipefail
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$HERE/../../.." && pwd)
UP=${UOR_POD:-$ROOT/scripts/pod/uor-pod}
LOCAL=${M2_LOCAL:-$ROOT/local/m2-run}
W=/workspace/m2
step=${1:?usage: drive.sh put|sync|watch|pod POD ...}; POD=${2:?POD}; shift 2
ssh_pod() { "$UP" ssh "$POD" "$@"; }

do_sync() {
  mkdir -p "$LOCAL/out"
  # Final (renamed) files only; .part files are still being written.
  local list
  list=$(ssh_pod "cd $W/out 2>/dev/null && sha256sum shard-*.jsonl counts-*.jsonl 2>/dev/null | grep -v '\.part' || true")
  local need=()
  while read -r sum name; do
    [ -n "$name" ] || continue
    if [ ! -f "$LOCAL/out/$name" ] || [ "$(shasum -a 256 "$LOCAL/out/$name" | cut -d' ' -f1)" != "$sum" ]; then need+=("$name"); fi
  done <<< "$list"
  if [ ${#need[@]} -gt 0 ]; then
    ssh_pod "cd $W/out && tar cf - ${need[*]}" | tar xf - -C "$LOCAL/out"
  fi
  # Logs are small and still growing: copy them every time.
  ssh_pod "cd $W/out && tar cf - rate.jsonl segments.jsonl dropped.jsonl metrics.json 2>/dev/null || true" | tar xf - -C "$LOCAL/out" 2>/dev/null || true
  local bad=0
  while read -r sum name; do
    [ -n "$name" ] || continue
    [ "$(shasum -a 256 "$LOCAL/out/$name" | cut -d' ' -f1)" = "$sum" ] || { echo "MISMATCH $name"; bad=1; }
  done <<< "$list"
  echo "$(date -u +%FT%TZ) synced $(echo "$list" | grep -c . || true) files (${#need[@]} new), mismatches=$bad; $(tail -1 "$LOCAL/out/rate.jsonl" 2>/dev/null)"
  return $bad
}

case $step in
put)
  T=$(mktemp -d "${TMPDIR:-/tmp}/m2put.XXXX")
  mkdir -p "$T/instrument"
  cp -R "$HERE/instrument/Cargo.toml" "$HERE/instrument/Cargo.lock" "$HERE/instrument/src" "$T/instrument/"
  cp "$HERE/pod.sh" "$T/"
  for f in bank.jsonl bank.jsonl.manifest.json model.json machine.json engine.json; do
    [ -f "$LOCAL/$f" ] && cp "$LOCAL/$f" "$T/"
  done
  tar cf - -C "$T" . | ssh_pod "mkdir -p $W && tar xf - -C $W && chmod +x $W/pod.sh && ls $W"
  rm -rf "$T"
  ;;
sync) do_sync ;;
watch)
  while true; do
    do_sync || true
    if ssh_pod "kill -0 \$(cat $W/run.pid 2>/dev/null) 2>/dev/null"; then sleep 600; continue; fi
    # The run has stopped: one last sync after the shard is finalized, then exit.
    sleep 30; do_sync && { echo "run stopped; all finished shards copied"; exit 0; }
    echo "final sync had mismatches"; exit 1
  done
  ;;
pod) ssh_pod "bash $W/pod.sh $*" ;;
*) echo "unknown step $step" >&2; exit 1 ;;
esac
