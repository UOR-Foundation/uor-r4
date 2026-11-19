#!/usr/bin/env bash
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd -P)
[[ $# == 1 ]]
mkdir "$1"
cd "$1"
jq -n -f "$here/finite.jq" > fixtures.json
for key in fg extra hg fb hb; do jq ".$key" fixtures.json > "$key"; done
for split in f h; do
    hash=$(shasum -a 256 "${split}b" | cut -d' ' -f1)
    jq --arg hash "$hash" ".${split}c | .blind_sha256=\$hash" fixtures.json > "${split}c"
done
check() { "$here/check-inputs.sh" fc "$1" fb hb fg extra hg "$2"; }
check hc pass.json
jq -e '.matched.confirmed_structural_ceiling==8 and .fit.original_literal_reachable==88' pass.json
# One changed candidate class breaks the signature equality only.
jq '.rows[0].assertions[0].legacy_candidates[0].slot_classes[0]=2' hc > shifted
if check shifted shifted.json; then exit 1; fi
jq -e '.matched.groups[0].distinct_structures==2 and .held.source_in_rank_domain==36' shifted.json
# Gold still writes from the original tape, but is absent from the ranked set.
jq '.rows[0].assertions[0].legacy_candidates[0].writer_reachable=false' hc > missing
if check missing missing.json; then exit 1; fi
jq -e '.held.original_literal_reachable==36 and .held.source_in_rank_domain==35' missing.json
