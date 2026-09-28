#!/usr/bin/env python3
"""Zero-Multiplier WebAssembly Serving Static Disassembly Auditor.

Verifies D11 and Milestone M1/M2 architectural invariants in compiled WebAssembly binaries:
- Strictly 0 floating-point instructions (Class III: f32.*, f64.*, and float conversions)
- Strictly 0 hardware integer multiplier instructions (Class I: i32.mul, i64.mul)
- Strictly 0 hardware integer divider instructions (Class II: i32.div_*, i64.div_*, rem_*)
across all compiled numerical serving symbols of `uor_r4_integer.wasm`.

Usage:
    python3 scripts/audit_zero_matmul_wasm.py [path_to_wasm] [--tap]
"""

import argparse
import hashlib
import os
import re
import subprocess
import sys


DEFAULT_WASM_PATH = "/Volumes/UOR-Workspace/BuildCaches/anti-gravity/wasm32-unknown-unknown/release/uor_r4_integer.wasm"


def get_artifact_metadata(path):
    sha256 = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            sha256.update(chunk)
    artifact_hash = sha256.hexdigest()

    commit = "unknown"
    try:
        res = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        commit = res.stdout.strip()
    except Exception:
        pass
    return artifact_hash, commit


def read_leb128_u(data, pos):
    val = 0
    shift = 0
    while True:
        b = data[pos]
        pos += 1
        val |= (b & 0x7F) << shift
        shift += 7
        if not (b & 0x80):
            break
    return val, pos


def read_leb128_s(data, pos):
    val = 0
    shift = 0
    while True:
        b = data[pos]
        pos += 1
        val |= (b & 0x7F) << shift
        shift += 7
        if not (b & 0x80):
            if b & 0x40:
                val |= -(1 << shift)
            break
    return val, pos


# WASM Opcode definitions
I32_MUL = 0x6C
I64_MUL = 0x7E
I32_DIV_S = 0x6D
I32_DIV_U = 0x6E
I32_REM_S = 0x6F
I32_REM_U = 0x70
I64_DIV_S = 0x7F
I64_DIV_U = 0x80
I64_REM_S = 0x81
I64_REM_U = 0x82

CLASS_I_OPS = {I32_MUL, I64_MUL}
CLASS_II_OPS = {
    I32_DIV_S,
    I32_DIV_U,
    I32_REM_S,
    I32_REM_U,
    I64_DIV_S,
    I64_DIV_U,
    I64_REM_S,
    I64_REM_U,
}
CLASS_III_FLOAT_OPS = (
    set(range(0x5B, 0x67))  # f32/f64 comparisons
    | set(range(0x8B, 0xA7))  # f32/f64 arithmetic
    | {
        0x2A,
        0x2B,  # f32/f64 load
        0x38,
        0x39,  # f32/f64 store
        0x43,
        0x44,  # f32/f64 const
        0xA8,
        0xA9,
        0xAA,
        0xAB,  # trunc float to int
        0xAE,
        0xAF,
        0xB0,
        0xB1,
        0xB2,
        0xB3,
        0xB4,
        0xB5,  # convert int to float
        0xB6,
        0xB7,
        0xB8,
        0xB9,
        0xBA,
        0xBB,  # demote / promote
        0xBC,
        0xBD,
        0xBE,
        0xBF,  # reinterpret
    }
)


def decode_function(data, start, end):
    pos = start
    num_locals, pos = read_leb128_u(data, pos)
    for _ in range(num_locals):
        cnt, pos = read_leb128_u(data, pos)
        val_type = data[pos]
        pos += 1

    opcodes = []
    while pos < end:
        op = data[pos]
        pos += 1
        opcodes.append(op)
        if op in (0x00, 0x01, 0x05, 0x0B, 0x0F):
            pass
        elif op in (0x02, 0x03, 0x04):
            bt = data[pos]
            pos += 1
            if bt not in (0x40, 0x7F, 0x7E, 0x7D, 0x7C):
                pass
        elif op in (0x0C, 0x0D):
            _, pos = read_leb128_u(data, pos)
        elif op == 0x0E:
            count, pos = read_leb128_u(data, pos)
            for _ in range(count):
                _, pos = read_leb128_u(data, pos)
            _, pos = read_leb128_u(data, pos)
        elif op == 0x10:
            _, pos = read_leb128_u(data, pos)
        elif op == 0x11:
            _, pos = read_leb128_u(data, pos)
            pos += 1
        elif op in (0x1A, 0x1B):
            pass
        elif op in (0x20, 0x21, 0x22, 0x23, 0x24):
            _, pos = read_leb128_u(data, pos)
        elif 0x28 <= op <= 0x3E:
            _, pos = read_leb128_u(data, pos)
            _, pos = read_leb128_u(data, pos)
        elif op in (0x3F, 0x40):
            pos += 1
        elif op == 0x41:
            _, pos = read_leb128_s(data, pos)
        elif op == 0x42:
            _, pos = read_leb128_s(data, pos)
        elif op == 0x43:
            pos += 4
        elif op == 0x44:
            pos += 8
        elif op == 0xFC:
            sub_op, pos = read_leb128_u(data, pos)
            if sub_op in (8, 9):
                _, pos = read_leb128_u(data, pos)
                if sub_op == 8:
                    pos += 1
            elif sub_op in (10, 11):
                pos += 2
        elif op == 0xFD:
            sub_op, pos = read_leb128_u(data, pos)
            if 0 <= sub_op <= 11:
                _, pos = read_leb128_u(data, pos)
                _, pos = read_leb128_u(data, pos)
            elif sub_op in (12, 13):
                pos += 16
            elif sub_op in (21, 22, 23, 24, 25, 26, 27, 28):
                _, pos = read_leb128_u(data, pos)
                _, pos = read_leb128_u(data, pos)
                pos += 1
            elif sub_op in (14, 15, 16, 17, 18, 19, 20):
                pos += 16
    return opcodes


# Mandatory serving symbols to audit in the compiled WASM binary
MANDATORY_WASM_SYMBOLS = [
    {
        "name": "uor_wasm_version",
        "pattern": "uor_wasm_version",
        "description": "API Version identifier export",
    },
    {
        "name": "uor_wasm_state_dim",
        "pattern": "uor_wasm_state_dim",
        "description": "Native width-576 dimension export",
    },
    {
        "name": "uor_wasm_vocab_size",
        "pattern": "uor_wasm_vocab_size",
        "description": "Vocabulary size 4096 export",
    },
    {
        "name": "uor_wasm_context_capacity",
        "pattern": "uor_wasm_context_capacity",
        "description": "Active context capacity export",
    },
    {
        "name": "uor_wasm_exact_min_p_threshold",
        "pattern": "uor_wasm_exact_min_p_threshold",
        "description": "Shift-add Min-P threshold calculation",
    },
    {
        "name": "uor_wasm_hopf_project_x",
        "pattern": "uor_wasm_hopf_project_x",
        "description": "Discrete Hopf base coordinate X projection",
    },
    {
        "name": "uor_wasm_hopf_project_y",
        "pattern": "uor_wasm_hopf_project_y",
        "description": "Discrete Hopf base coordinate Y projection",
    },
    {
        "name": "uor_wasm_hopf_project_z",
        "pattern": "uor_wasm_hopf_project_z",
        "description": "Discrete Hopf base coordinate Z projection",
    },
    {
        "name": "uor_wasm_step_zeta_phase_scalar",
        "pattern": "uor_wasm_step_zeta_phase_scalar",
        "description": "Scalar zeta phase shift-add progression",
    },
    {
        "name": "uor_wasm_galois_lfsr_step",
        "pattern": "uor_wasm_galois_lfsr_step",
        "description": "Galois LFSR coordinate step",
    },
    {
        "name": "uor_wasm_compute_turn_prime_signature",
        "pattern": "uor_wasm_compute_turn_prime_signature",
        "description": "Turn prime signature hash calculation",
    },
    {
        "name": "uor_wasm_atan2_q30",
        "pattern": "uor_wasm_atan2_q30",
        "description": "CORDIC Q30 arctangent projection",
    },
    {
        "name": "uor_wasm_score_token_salience_default_key",
        "pattern": "uor_wasm_score_token_salience_default_key",
        "description": "Token salience scoring with zero multipliers",
    },
    {
        "name": "IntegerModel::project_vocab",
        "pattern": "IntegerModel13project_vocab",
        "description": "Vocab projection via shift-and-add rows",
    },
    {
        "name": "IntegerModel::project_vocab_with_products",
        "pattern": "IntegerModel27project_vocab_with_products",
        "description": "Vocab projection with 4-row blocked product reuse",
    },
    {
        "name": "low_bit_dot_8_pair_tables",
        "pattern": "low_bit_dot_8_pair_tables",
        "description": "8-pair lookup table inner product kernel",
    },
    {
        "name": "low_bit_dot_4_contiguous_wide_pair_tables",
        "pattern": "low_bit_dot_4_contiguous_wide_pair_t",
        "description": "4-row contiguous wide pair table kernel",
    },
    {
        "name": "packed_rows::low_bit_dot",
        "pattern": "packed_rows11low_bit_dot",
        "description": "Low-bit dot product over packed signed-4 rows",
    },
    {
        "name": "sampling::exact_min_p_threshold",
        "pattern": "sampling21exact_min_p_threshold",
        "description": "Exact shift-add Min-P threshold in sampling module",
    },
    {
        "name": "math::atan2_q30",
        "pattern": "math9atan2_q30",
        "description": "CORDIC fixed-point arctangent",
    },
]


def main():
    parser = argparse.ArgumentParser(
        description="Audit WASM binary for zero hardware multiplier/divider/float instructions under D11."
    )
    parser.add_argument("wasm_path", nargs="?", default=DEFAULT_WASM_PATH)
    parser.add_argument("--tap", action="store_true", help="Output in TAP 13 format")
    args = parser.parse_args()

    wasm_path = os.path.abspath(args.wasm_path)
    if not os.path.exists(wasm_path):
        print(f"Error: WASM file not found: {wasm_path}", file=sys.stderr)
        sys.exit(1)

    artifact_hash, commit = get_artifact_metadata(wasm_path)

    with open(wasm_path, "rb") as f:
        data = f.read()

    if data[:4] != b"\x00asm":
        print(f"Error: Invalid WASM magic header in {wasm_path}", file=sys.stderr)
        sys.exit(1)

    # Parse sections
    pos = 8
    code_sec = None
    export_sec = None
    while pos < len(data):
        sec_id = data[pos]
        pos += 1
        sec_len, pos = read_leb128_u(data, pos)
        sec_end = pos + sec_len
        if sec_id == 7:
            export_sec = (pos, sec_len)
        elif sec_id == 10:
            code_sec = (pos, sec_len)
        pos = sec_end

    if not code_sec:
        print("Error: No code section found in WASM", file=sys.stderr)
        sys.exit(1)

    exports = {}
    if export_sec:
        p, sec_len = export_sec
        count, p = read_leb128_u(data, p)
        for _ in range(count):
            name_len, p = read_leb128_u(data, p)
            name = data[p : p + name_len].decode("utf-8", errors="replace")
            p += name_len
            kind = data[p]
            p += 1
            idx, p = read_leb128_u(data, p)
            if kind == 0:
                exports.setdefault(idx, []).append(name)

    p, sec_len = code_sec
    fn_count, p = read_leb128_u(data, p)

    fn_bodies = []
    for _ in range(fn_count):
        body_size, p = read_leb128_u(data, p)
        body_end = p + body_size
        fn_bodies.append((p, body_end))
        p = body_end

    # Match mandatory symbols
    results = []
    all_passed = True

    for item in MANDATORY_WASM_SYMBOLS:
        sym_name = item["name"]
        pattern = item["pattern"]
        matched_fn_indices = []

        for fn_idx, (b_start, b_end) in enumerate(fn_bodies):
            names = exports.get(fn_idx, [])
            if any(pattern in n for n in names):
                matched_fn_indices.append(fn_idx)

        if not matched_fn_indices:
            results.append({
                "name": sym_name,
                "status": "NOT_FOUND",
                "details": f"Symbol pattern '{pattern}' not found in WASM exports",
                "muls": 0,
                "divs": 0,
                "floats": 0,
            })
            all_passed = False
            continue

        total_muls = 0
        total_divs = 0
        total_floats = 0
        fn_insn_counts = []

        for fn_idx in matched_fn_indices:
            b_start, b_end = fn_bodies[fn_idx]
            ops = decode_function(data, b_start, b_end)
            muls = sum(1 for op in ops if op in CLASS_I_OPS)
            divs = sum(1 for op in ops if op in CLASS_II_OPS)
            floats = sum(1 for op in ops if op in CLASS_III_FLOAT_OPS)

            total_muls += muls
            total_divs += divs
            total_floats += floats
            fn_insn_counts.append(len(ops))

        passed = (total_muls == 0) and (total_divs == 0) and (total_floats == 0)
        if not passed:
            all_passed = False

        results.append({
            "name": sym_name,
            "status": "PASS" if passed else "FAIL",
            "fn_count": len(matched_fn_indices),
            "total_insns": sum(fn_insn_counts),
            "muls": total_muls,
            "divs": total_divs,
            "floats": total_floats,
            "details": f"{sum(fn_insn_counts)} instructions across {len(matched_fn_indices)} fn(s)",
        })

    if args.tap:
        print("TAP version 13")
        print(f"1..{len(results)}")
        print(f"# Artifact: {wasm_path}")
        print(f"# SHA-256: {artifact_hash}")
        print(f"# Commit: {commit}")
        for i, res in enumerate(results, 1):
            if res["status"] == "PASS":
                print(
                    f"ok {i} - {res['name']}: 0 mul, 0 div, 0 float ({res['details']})"
                )
            else:
                print(
                    f"not ok {i} - {res['name']}: {res['status']} (mul={res['muls']}, div={res['divs']}, float={res['floats']})"
                )
    else:
        print("=" * 80)
        print("UOR-R4 WebAssembly Zero-Multiplier Serving Auditor (D11)")
        print(f"Artifact: {wasm_path}")
        print(f"SHA-256:  {artifact_hash}")
        print(f"Commit:   {commit}")
        print("=" * 80)
        for res in results:
            status_str = f"[{res['status']}]"
            print(
                f"{status_str:<10} {res['name']:<45} | mul: {res['muls']:2d} | div: {res['divs']:2d} | float: {res['floats']:2d} | {res['details']}"
            )
        print("-" * 80)
        if all_passed:
            print("AUDIT VERDICT: PASS (100% compliant with D11 zero-multiplier, zero-divider, zero-float)")
        else:
            print("AUDIT VERDICT: FAIL (non-compliant instructions detected in numerical serving symbols)")

    sys.exit(0 if all_passed else 1)


if __name__ == "__main__":
    main()
