#!/bin/bash
set -euo pipefail
cd /workspace/orderfree-state-21-uort
sha256sum -c stage.sha256
sha256sum -c known-inputs.sha256
printf "%s\n" "8e236d53bd40e95c9225b74353d056c43a7a4d7977747d67e5f820194f5a1207  FIT-parent.md" | sha256sum -c
printf "%s\n" "a262e358ef40c1953a4a279153286e10d9f74b4212fc120fbd8d686043bd4951  question-causal-state-cuda" | sha256sum -c
test "${CUDA_VISIBLE_DEVICES:-}" = 0
test "$(hostname)" = 0150a7a76615
test "$(nvidia-smi -i 0 --query-gpu=uuid --format=csv,noheader)" = GPU-f9afc849-5fb5-d7c6-c8d5-3154e9a2bd25
test -z "$(nvidia-smi -i 0 --query-compute-apps=pid --format=csv,noheader,nounits)"
test "$(nvidia-smi -i 0 --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 1024
(cd finite-run-1; sha256sum -c receipts.sha256)
jq -e '.all_raw_pre_gate_reads_match_host and .all_knockout_reads_match_ordinary_evaluation and .uort_first_last_onsets_alignment and .conflicting_label_abstention and .different_labels_on_state_donation' finite-run-1/finite.json
cd fit-run-1
nvidia-smi -i 0 --query-gpu=uuid,name,driver_version,memory.free,memory.used,utilization.gpu --format=csv > gpu-before.csv
/usr/bin/time -v -o fit.time ../question-causal-state-cuda fit ../order-free-seed1.d11 ../train-tokens.u16 ../train-mask.u8 4096 train-index.json > fit.stdout 2> fit.stderr
jq '{artifact, train_tokens, train_mask, selected, labeled, prototypes: (.prototypes|length)}' train-index.json > train-index-summary.json
nvidia-smi -i 0 --query-gpu=uuid,name,driver_version,memory.free,memory.used,utilization.gpu --format=csv > gpu-after.csv
sha256sum train-index.json train-index-summary.json fit.time fit.stdout fit.stderr gpu-before.csv gpu-after.csv run.sh > receipts.sha256
