#!/usr/bin/env bash
# Verify graded inputs, both native orientations and both cold/device repeats before releasing the key.
set -euo pipefail
umask 077
test "$(hostname -s)" = wick
root=$(realpath "$1")
split=$(realpath "$2")
device=$(realpath "$3")
primary=$(realpath "$4")
cold=$(realpath "$5")
frozen=$(realpath "$6")
tools_manifest=$(realpath "$7")
expected_tools_sha=$8
out=$9
printf '%s  %s\n' "$expected_tools_sha" "$tools_manifest" | sha256sum --strict -c -
(cd "$root" && sha256sum --strict --status -c "$tools_manifest")
mkdir "$out"
out=$(realpath "$out")
(cd "$device" && sha256sum --strict --status -c results.sha256)
for geometry in fact-prefix candidate-prefix; do
    (cd "$split/$geometry" && sha256sum --strict --status -c packets.sha256 && sha256sum --strict --status -c indexes.sha256)
done
for dir in "$primary" "$cold"; do
    (cd "$dir" && sha256sum --strict --status -c captures.sha256)
done
cmp "$primary/captures.sha256" "$cold/captures.sha256"
for prefix in fact candidate; do
    geometry=$prefix-prefix
    for mode in ordinary supplied; do
        packet=$mode
        if [[ $mode == supplied ]]; then packet=diagnostic; fi
        for pass in primary cold; do
            capture=$primary
            if [[ $pass == cold ]]; then capture=$cold; fi
            base="$out/$prefix-$mode-$pass"
            "$root/device/paired-value-decode" "$device/$geometry/$pass/$packet.out" > "$base.decoded.json"
            "$root/grade/target/release/d35-parity" "$split/$geometry/$packet.index.json" "$base.decoded.json" \
                "$capture/$mode/${prefix}_true" "$capture/$mode/${prefix}_reversed" > "$base.parity.json"
        done
    done
done
(cd "$out" && sha256sum *.parity.json *.decoded.json > parity.sha256)
# The key lives outside serving, projection and CUDA inputs. It can enter only the following offline grade.
(cd "$frozen/offline" && sha256sum --strict --status -c manifest.sha256)
jq '{baseline:.parent_sha256, fact_true:.fact_carrier_sha256, fact_reversed:.fact_carrier_sha256,
    candidate_true:.candidate_carrier_sha256,candidate_reversed:.candidate_carrier_sha256}' "$primary/go.json" > "$out/arm-hashes.json"
grade="$root/grade/target/release"
"$grade/d35-value-grade" "$frozen/offline/key.json" "$primary" "$cold" "$out/arm-hashes.json" > "$out/graded.json"
"$grade/d35-summary" "$out/graded.json" > "$out/summary.json"
(cd "$out" && sha256sum arm-hashes.json parity.sha256 graded.json summary.json > results.sha256)
