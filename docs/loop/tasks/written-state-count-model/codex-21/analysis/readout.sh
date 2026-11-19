#!/usr/bin/env bash
# Work on a complete staged result. Data stays outside the source tree.
set -euo pipefail
[[ $# == 5 ]] || {
    echo "usage: readout.sh RUN PANEL TOKENIZER PIECES_BIN NEW_OUT" >&2
    exit 2
}
run=$1 panel=$2 tokenizer=$3 pieces=$4 out=$5
src=$(cd "$(dirname "$0")" && pwd -P)
[[ "$(cat "$run/status")" == "done 0" ]]
(cd "$run" && shasum -a 256 --status --check raw.sha256 && shasum -a 256 --status --check cold.sha256)
(cd "$panel" && shasum -a 256 --status --check INPUTS.sha256)
seed=$(jq -er '.seed|select(.==1 or .==2)' "$run/spec.json")
held="$panel/seed$seed/held"
pin() { shasum -a 256 "$1" | cut -d' ' -f1; }
[[ "$(pin "$held/grades.json")" == "$(jq -er .files.labels.sha256 "$run/spec.json")" ]]
[[ "$(pin "$held/blind.json")" == "$(jq -er .files.blind.sha256 "$run/spec.json")" ]]
mkdir "$out"
"$pieces" "$tokenizer" "$(jq -er .files.tokenizer.sha256 "$run/spec.json")" > "$out/pieces.json"
writer_args=()
observed_args=()
source_args=()
audit_args=()
for spec in reference:reference clause:treatment supplied:supplied veto:supplied-veto wrong_role:wrong-role no_read:no-read; do
    key=${spec%:*} file=${spec#*:}
    writer_args+=(--slurpfile "$key" "$run/grades/$file-writer.json")
    observed_args+=(--slurpfile "$key" "$run/grades/$file.json")
    if [[ "$key" != no_read ]]; then
        source_args+=(--slurpfile "$key" "$run/modes/$file/sources.json")
        audit_args+=(--slurpfile "$key" "$run/arms/$file-source-audit.json")
    fi
done
jq -ne "${writer_args[@]}" '$ARGS.named|map_values(.[0])' > "$out/writers.json"
jq -ne "${observed_args[@]}" '$ARGS.named|map_values(.[0])' > "$out/observations.json"
jq -ne "${source_args[@]}" '$ARGS.named|map_values(.[0])' > "$out/traces.json"
jq -ne "${audit_args[@]}" '$ARGS.named|map_values(.[0])' > "$out/audits.json"
jq -ne --slurpfile gold "$run/grades/forced-gold.json" --slurpfile wrong "$run/grades/forced-wrong.json" \
    --slurpfile key "$run/grades/wrong-key.json" --slurpfile roles "$run/capture/fixed-roles-verified.json" \
    --slurpfile passed "$run/grades/controls-verified.json" --arg raw "$(pin "$run/raw.sha256")" \
    --arg cold "$(pin "$run/cold.sha256")" --arg panel "$(pin "$panel/INPUTS.sha256")" '
    select($roles[0]==true and $passed[0]==true) |
    {raw_manifest:$raw,cold_manifest:$cold,panel_manifest:$panel, fixed_roles_verified:$roles[0],
     controls_verified:$passed[0], forced_gold:$gold[0].counts, forced_wrong:$wrong[0].counts, wrong_key:$key[0].counts}
' > "$out/verification.json"
jq -ne --argjson seed "$seed" --slurpfile labels "$held/grades.json" --slurpfile blind "$held/blind.json" \
   --slurpfile pieces "$out/pieces.json" --slurpfile writers "$out/writers.json" \
   --slurpfile observations "$out/observations.json" --slurpfile traces "$out/traces.json" \
   --slurpfile audits "$out/audits.json" \
   --slurpfile verification "$out/verification.json" -f "$src/readout.jq" > "$out/readout.json"
jq -f "$src/readout-compact.jq" "$out/readout.json" > "$out/compact.json"

jq -r -f "$src/readout-markdown.jq" "$out/readout.json" > "$out/readout.md"
