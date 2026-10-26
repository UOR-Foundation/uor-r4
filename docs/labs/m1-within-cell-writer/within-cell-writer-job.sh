#!/usr/bin/env bash
set -euo pipefail

# Within-Cell Writer Qualification, Stage T Decomposition & Zero-Update B0 Job (#3267)
# Runs on 1x RTX 5090 (sm_120) leased via scripts/pod/uor-pod.

WORK_ROOT="/workspace/within-cell-writer"
DATA_ROOT="/workspace/uor-r4/local/driver/data"
STAGE_T_ARCHIVE_DIR="/workspace/stage-t-archive"
STAGE_T_DIR="/workspace/stage-t-extracted/stage-t-out"
OUT1="${WORK_ROOT}/build1-out"
OUT2="${WORK_ROOT}/build2-out"
BIN_DIR="${WORK_ROOT}/bin"

mkdir -p "${WORK_ROOT}" "${DATA_ROOT}" "${STAGE_T_ARCHIVE_DIR}" "${OUT1}" "${OUT2}" "${BIN_DIR}"

echo "[$(date -u +%FT%TZ)] Fetching driver corpus & address-structured-v1 from uor-store..."
if [ ! -f "${DATA_ROOT}/mw-balp/train/tokens.u16" ]; then
    /root/uor-store get inputs/driver-data/ "${DATA_ROOT}"
fi
(cd "${DATA_ROOT}" && sha256sum -c SHA256SUMS)

echo "[$(date -u +%FT%TZ)] Fetching sealed Stage T archive from uor-store..."
if [ ! -f "${STAGE_T_ARCHIVE_DIR}/stage-t-20261025.tar.gz" ]; then
    /root/uor-store get results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/ "${STAGE_T_ARCHIVE_DIR}"
fi
(cd "${STAGE_T_ARCHIVE_DIR}" && sha256sum -c SHA256SUMS)
mkdir -p /workspace/stage-t-extracted
tar -xzf "${STAGE_T_ARCHIVE_DIR}/stage-t-20261025.tar.gz" -C /workspace/stage-t-extracted

SRC_DIR=$(ls -d /root/build/src-* | head -n 1)
echo "[$(date -u +%FT%TZ)] Building within-cell-writer-run (Build 1 & Build 2) from ${SRC_DIR}..."
cd "${SRC_DIR}"
export PATH="/root/.cargo/bin:${PATH}"
export UOR_NVCC_ARCH=compute_120

CARGO_TARGET_DIR=/root/build/target-wcw-1 cargo build --release --locked -p uor-r4-discrete --features cuda --bin within-cell-writer-run
cp /root/build/target-wcw-1/release/within-cell-writer-run "${BIN_DIR}/within-cell-writer-run-build1"

CARGO_TARGET_DIR=/root/build/target-wcw-2 cargo build --release --locked -p uor-r4-discrete --features cuda --bin within-cell-writer-run
cp /root/build/target-wcw-2/release/within-cell-writer-run "${BIN_DIR}/within-cell-writer-run-build2"

sha256sum "${BIN_DIR}/within-cell-writer-run-build1" "${BIN_DIR}/within-cell-writer-run-build2" > "${BIN_DIR}/SHA256SUMS"
cat "${BIN_DIR}/SHA256SUMS"

echo "[$(date -u +%FT%TZ)] Running Build 1: qualify..."
"${BIN_DIR}/within-cell-writer-run-build1" qualify "${OUT1}/within-cell-qualification.json"

echo "[$(date -u +%FT%TZ)] Running Build 1: decompose-stage-t..."
"${BIN_DIR}/within-cell-writer-run-build1" decompose-stage-t "${STAGE_T_DIR}" "${DATA_ROOT}" "${OUT1}/stage-t-decomposition.json"

echo "[$(date -u +%FT%TZ)] Running Build 1: b0..."
"${BIN_DIR}/within-cell-writer-run-build1" b0 "${DATA_ROOT}" "${OUT1}/within-cell-qualification.json" "${OUT1}/stage-t-decomposition.json" "${OUT1}/b0-grade.json"

echo "[$(date -u +%FT%TZ)] Running Build 2: qualify & b0 determinism replication..."
"${BIN_DIR}/within-cell-writer-run-build2" qualify "${OUT2}/within-cell-qualification.json"
cp "${OUT1}/stage-t-decomposition.json" "${OUT2}/stage-t-decomposition.json"
"${BIN_DIR}/within-cell-writer-run-build2" b0 "${DATA_ROOT}" "${OUT1}/within-cell-qualification.json" "${OUT1}/stage-t-decomposition.json" "${OUT2}/b0-grade.json"

(cd "${WORK_ROOT}" && find build1-out build2-out bin -type f -exec sha256sum {} + | sort > SHA256SUMS)
echo "WITHIN_CELL_WRITER_JOB_COMPLETE" > "${WORK_ROOT}/EXIT_OK"
echo "[$(date -u +%FT%TZ)] Job complete."
