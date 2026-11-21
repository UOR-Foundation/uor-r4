#!/bin/bash
set -euo pipefail
cd /workspace/orderfree-state-21-uort
sha256sum -c stage.sha256
sha256sum -c known-inputs.sha256
printf "%s\n" "a262e358ef40c1953a4a279153286e10d9f74b4212fc120fbd8d686043bd4951  question-causal-state-cuda" | sha256sum -c
test "${CUDA_VISIBLE_DEVICES:-}" = 0
test "$(hostname)" = 0150a7a76615
test "$(nvidia-smi -i 0 --query-gpu=uuid --format=csv,noheader)" = GPU-f9afc849-5fb5-d7c6-c8d5-3154e9a2bd25
test -z "$(nvidia-smi -i 0 --query-compute-apps=pid --format=csv,noheader,nounits)"
test "$(nvidia-smi -i 0 --query-gpu=memory.free --format=csv,noheader,nounits)" -ge 1024
cd finite-run-1
nvidia-smi -i 0 --query-gpu=uuid,name,driver_version,memory.free,memory.used,utilization.gpu --format=csv > gpu-before.csv
/usr/bin/time -v -o finite.time ../question-causal-state-cuda finite ../order-free-seed1.d11 finite.json > finite.stdout 2> finite.stderr
nvidia-smi -i 0 --query-gpu=uuid,name,driver_version,memory.free,memory.used,utilization.gpu --format=csv > gpu-after.csv
sha256sum finite.json finite.time finite.stdout finite.stderr gpu-before.csv gpu-after.csv run.sh > receipts.sha256
