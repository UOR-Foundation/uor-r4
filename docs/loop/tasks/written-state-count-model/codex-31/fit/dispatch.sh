#!/usr/bin/env bash
set -euo pipefail
umask 077

if [[ $# != 7 ]]; then
    echo "usage: dispatch.sh FIT_HOST FIT_DEVICE SEALED_CR_DIR TRUNK COMPACT_COUNTS COMPACT_SHA256 NEW_OUT" >&2
    exit 2
fi
host=$1 device=$2 cr=$3 trunk=$4 counts=$5 count_sha=$6 out=$7
test ! -e "$out"
test ! -e "$out-resources"
sha256sum -c GO.sha256
grep -q '^TEST FITNESS: FIT' FIT.md
test "$(sha256sum "$counts" | cut -d ' ' -f1)" = "$count_sha"
test "$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader)" = 12.0
test "$(nvidia-smi --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 20480
test "$(awk '/^MemAvailable:/ {print $2}' /proc/meminfo)" -ge 25165824
test "$(df -Pk "$(dirname "$out")" | awk 'NR == 2 {print $4}')" -ge 6291456
mkdir "$out-resources"
/usr/bin/time -v -o "$out-resources/prepare.resources.txt" \
    "$host" prepare "$cr" "$trunk" "$counts" "$count_sha" "$out"
printf '%s  %s\n' "$(jq -er '.rows_sha256' "$out/prepare.json")" "$out/train.rows" | sha256sum -c
printf '%s  %s\n' "$(jq -er '.probe_ordinals_sha256' "$out/prepare.json")" "$out/probe-ordinals.u64" | sha256sum -c
/usr/bin/time -v -o "$out-resources/fit.resources.txt" \
    "$device" "$out/train.rows" "$out/fit.trace" "$out/probe-ordinals.u64" "$out/probe-masses.u64"
"$host" seal "$out" | tee "$out-resources/seal.json"
sha256sum "$out"/* "$out-resources"/{prepare.resources.txt,fit.resources.txt,seal.json} > "$out-resources/results.sha256"
