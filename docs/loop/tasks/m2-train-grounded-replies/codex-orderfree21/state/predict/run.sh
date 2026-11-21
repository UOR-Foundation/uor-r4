#!/bin/bash
set -euo pipefail
cd /workspace/orderfree-state-21-uort
sha256sum -c stage.sha256
sha256sum -c known-inputs.sha256
(cd finite-run-1; sha256sum -c receipts.sha256)
(cd fit-run-1; sha256sum -c receipts.sha256)
test "${CUDA_VISIBLE_DEVICES:-}" = 0
test "$(hostname)" = 0150a7a76615
test "$(nvidia-smi -i 0 --query-gpu=uuid --format=csv,noheader)" = GPU-f9afc849-5fb5-d7c6-c8d5-3154e9a2bd25
test -z "$(nvidia-smi -i 0 --query-compute-apps=pid --format=csv,noheader,nounits)"
test "$(nvidia-smi -i 0 --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 1024
cd predict-run-1
sha256sum -c GO.sha256
jq -r "[.binary, .artifact, .index, .prompts, .fit] | .[]" GO.json | paste -d " " - <(printf " %s\n" ../question-causal-state-cuda ../order-free-seed1.d11 ../fit-run-1/train-index.json prompts.json ../FIT-parent.md) | sha256sum -c
jq -e '. as $root | length == 160 and all(.[]; keys == ["id", "prompt"]) and ([.[] | .id] | unique | length == 160)' prompts.json
nvidia-smi -i 0 --query-gpu=uuid,name,driver_version,memory.free,memory.used,utilization.gpu --format=csv > gpu-before.csv
/usr/bin/time -v -o predict.time ../question-causal-state-cuda predict ../order-free-seed1.d11 ../fit-run-1/train-index.json prompts.json 48 predictions.json > predict.stdout 2> predict.stderr
jq -e --slurpfile p prompts.json 'length == 160' predictions.json
jq -en --slurpfile p prompts.json --slurpfile r predictions.json '
  all(range(0; 160); . as $i | (($i / 2 | floor) * 2 + (1 - ($i % 2))) as $donor |
  $p[0][$i].id == $r[0][$i].id and $p[0][$i].prompt == $r[0][$i].prompt and $p[0][$donor].id == $r[0][$i].donor_id and $r[0][$donor].trunk_state == $r[0][$i].donor_state)
'
sha256sum predictions.json > predictions.sha256
nvidia-smi -i 0 --query-gpu=uuid,name,driver_version,memory.free,memory.used,utilization.gpu --format=csv > gpu-after.csv
sha256sum GO.json GO.sha256 prompts.json predictions.json predictions.sha256 predict.time predict.stdout predict.stderr gpu-before.csv gpu-after.csv run.sh > receipts.sha256
