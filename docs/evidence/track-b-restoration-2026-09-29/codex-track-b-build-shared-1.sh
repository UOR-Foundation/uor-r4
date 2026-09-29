#!/bin/bash
set -uo pipefail
cd /Users/casey.allard/.codex/worktrees/track-b-conversion/uor-r4 || exit 1
artifact_base=/Volumes/UOR-Workspace/uor-r4-lab/codex-track-b-conversion-20260929
cache_root=/Volumes/UOR-Workspace/BuildCaches/fourth-lab-dialogue-debug-20260927-1/target
slot_path=/Volumes/UOR-Workspace/locks/model-slot.json
if pgrep -x cargo >/dev/null; then echo 'BUSY: another Cargo process is live'; exit 75; fi
available_kib=$(/bin/df -Pk /Volumes/UOR-Workspace | awk 'NR==2 {print $4}') || exit 75
if ! [[ "$available_kib" =~ ^[0-9]+$ ]] || [ "$available_kib" -lt 25657344 ]; then echo "STORAGE_NOT_ADMITTED: $available_kib KiB"; exit 75; fi
if ! (set -o noclobber; printf '{"lab":"Codex Track B focused checks","branch":"codex/track-b-conversion","pid":%s,"started_utc":"%s","expected_end_utc":"%s","threads":2,"rss_cap_gb":4}\n' "$$" "$(date -u +%FT%TZ)" "$(date -u -v+20M +%FT%TZ)" > "$slot_path"); then echo 'BUSY: model slot owned'; exit 75; fi
own_slot_pid=$$
time_pid=''
cache_watch_pid=''
wall_watch_pid=''
stop_owned_build() {
  if [ -n "$time_pid" ] && kill -0 "$time_pid" 2>/dev/null; then
    cargo_pid=$(pgrep -P "$time_pid" | head -1 || true)
    if [ -n "$cargo_pid" ]; then pkill -TERM -P "$cargo_pid" 2>/dev/null || true; kill -TERM "$cargo_pid" 2>/dev/null || true; fi
    kill -TERM "$time_pid" 2>/dev/null || true
  fi
}
cleanup() {
  for watcher in "$cache_watch_pid" "$wall_watch_pid"; do
    if [ -n "$watcher" ]; then pkill -TERM -P "$watcher" 2>/dev/null || true; kill -TERM "$watcher" 2>/dev/null || true; fi
  done
  if [ "$(jq -r '.pid' "$slot_path" 2>/dev/null)" = "$own_slot_pid" ]; then rm "$slot_path"; fi
}
trap 'stop_owned_build; cleanup' EXIT
set -o noclobber
exec 3> "$artifact_base/build-shared-1.log" || exit 1
source_revision=$(git rev-parse HEAD)
source_diff=$(git diff HEAD | shasum -a 256 | awk '{print $1}')
printf 'source_revision=%s\nsource_diff_sha256=%s\ncache_baseline_kib=7469744\ncache_cap_kib=7731888\navailable_before_kib=%s\n' "$source_revision" "$source_diff" "$available_kib" >&3
mkdir -p "$cache_root/codex-shared-tmp" || exit 1
start_seconds=$SECONDS
/usr/bin/time -l env TMPDIR="$cache_root/codex-shared-tmp" CARGO_TARGET_DIR="$cache_root" CARGO_BUILD_JOBS=2 RAYON_NUM_THREADS=2 TRACK_B_SOURCE_REVISION="$source_revision" TRACK_B_SOURCE_DIFF_SHA256="$source_diff" /Users/casey.allard/.cargo/bin/cargo test --offline --release -p uor-r4-training --features metal --lib track_b:: -- --test-threads=1 >&3 2>&1 &
time_pid=$!
(sleep 1200; printf 'BUILD_WALL_BOUND\n' >&3; stop_owned_build) &
wall_watch_pid=$!
(
 while kill -0 "$time_pid" 2>/dev/null; do
   if ! usage_kib=$(du -sk "$cache_root" | awk '{print $1}'); then printf 'CACHE_OBSERVATION_FAILED\n' >&3; stop_owned_build; break; fi
   printf 'cache_observed_kib=%s\n' "$usage_kib" >&3
   if ! [[ "$usage_kib" =~ ^[0-9]+$ ]] || [ "$usage_kib" -gt 7731888 ]; then printf 'CACHE_GROWTH_CAP_STOP\n' >&3; stop_owned_build; break; fi
   sleep 15
 done
) &
cache_watch_pid=$!
stop_reason=''
while kill -0 "$time_pid" 2>/dev/null; do
  if ! available_kib=$(/bin/df -Pk /Volumes/UOR-Workspace | awk 'NR==2 {print $4}'); then stop_reason='STORAGE_OBSERVATION_FAILED'; fi
  if ! [[ "$available_kib" =~ ^[0-9]+$ ]]; then stop_reason='STORAGE_OBSERVATION_FAILED'; elif [ "$available_kib" -lt 25296896 ]; then stop_reason='STORAGE_GUARD'; fi
  if [ -n "$stop_reason" ]; then printf '%s\n' "$stop_reason" >&3; stop_owned_build; break; fi
  sleep 2
done
wait "$time_pid"
exit_code=$?
cleanup
cache_watch_pid=''
wall_watch_pid=''
cache_after_kib=$(du -sk "$cache_root" | awk '{print $1}') || cache_after_kib=0
if [ "$cache_after_kib" -eq 0 ]; then stop_reason='CACHE_FINAL_OBSERVATION_FAILED'; exit_code=125; elif [ "$cache_after_kib" -gt 7731888 ]; then stop_reason='CACHE_GROWTH_CAP_EXCEEDED'; exit_code=125; fi
printf '{"source_revision":"%s","source_diff_sha256":"%s","exit_code":%s,"stop_reason":"%s","elapsed_s":%s,"cache_before_kib":7469744,"cache_after_kib":%s,"available_after_kib":%s}\n' "$source_revision" "$source_diff" "$exit_code" "$stop_reason" "$((SECONDS-start_seconds))" "$cache_after_kib" "${available_kib:-0}" > "$artifact_base/build-shared-1-exit.json"
cat "$artifact_base/build-shared-1.log"
cat "$artifact_base/build-shared-1-exit.json"
exit "$exit_code"
