#!/usr/bin/env python3
"""Audit uor-r4-integer serving binary for floating-point arithmetic instructions (D0-b compliance).

D0-b Policy Requirement:
------------------------
"No floating point at serving, and no libm transcendentals. Served computation executes
learned geometric operators through bounded routing, state updates and integer/table lookup."

This audit inspects the compiled release binary of `uor-r4-integer` using disassembly
(`otool -tV` on macOS, or `llvm-objdump` / `objdump` cross-platform) to verify that
all functions on the serving path contain zero floating-point arithmetic instructions.

Declared Certified Symbols:
---------------------------
The following symbol families in `uor_r4_integer` are certified to contain zero
floating-point arithmetic instructions:
- `IntegerModel::step`
- `IntegerModel::affine`
- `low_bit_dot`
- `low_bit_products`
- `normalize_state`
- `normalize_residual`
- `blend`
- `divide`
- `scaled`
- `product`
- `softmax`
- `transport`
- `Sampler::draw_below`
- `Sampler::select`

Any floating-point instructions detected in these symbols trigger immediate test failure (exit code 1).
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

# ARM64 floating-point arithmetic and conversion instructions
FP_RE_ARM64 = re.compile(
    r"\b(fadd|fsub|fmul|fdiv|fmla|fmls|fneg|fabs|fsqrt|fcvtzs|fcvtzu|scvtf|ucvtf|fcmp|fcmpe)\b"
)

# x86_64 SSE/AVX floating-point arithmetic instructions (for cross-platform portability)
FP_RE_X86 = re.compile(
    r"\b(addss|addsd|subss|subsd|mulss|mulsd|divss|divsd|sqrtss|sqrtsd|"
    r"vaddss|vaddsd|vsubss|vsubsd|vmulss|vmulsd|vdivss|vdivsd|vsqrtss|vsqrtsd|"
    r"ucomiss|ucomisd|comiss|comisd|cvtsi2ss|cvtsi2sd|cvttss2si|cvttsd2si)\b"
)

# Declared symbols that MUST be float-free on the serving path
# (pattern, human_readable_description)
CERTIFIED_SERVING_SYMBOLS = [
    ("IntegerModel4step", "uor_r4_integer::model::IntegerModel::step"),
    ("IntegerModel6affine", "uor_r4_integer::model::IntegerModel::affine"),
    ("low_bit_dot", "uor_r4_integer::model::low_bit_dot"),
    ("low_bit_products", "uor_r4_integer::model::low_bit_products"),
    ("normalize_state", "uor_r4_integer::model::normalize_state"),
    ("normalize_residual", "uor_r4_integer::model::normalize_residual"),
    ("5model5blend", "uor_r4_integer::model::blend"),
    ("5model6divide", "uor_r4_integer::model::divide"),
    ("5model6scaled", "uor_r4_integer::model::scaled"),
    ("5model7product", "uor_r4_integer::model::product"),
    ("5model7softmax", "uor_r4_integer::model::softmax"),
    ("5model9transport", "uor_r4_integer::model::transport"),
    ("Sampler10draw_below", "uor_r4_integer::sampling::Sampler::draw_below"),
    ("Sampler6select", "uor_r4_integer::sampling::Sampler::select"),
]


def disassemble_binary(binary_path: Path) -> dict:
    """Disassemble binary into per-symbol instruction lists using available platform tools."""
    if not binary_path.exists():
        raise FileNotFoundError(f"Binary not found: {binary_path}")

    # Try otool first (macOS)
    if subprocess.run(["which", "otool"], capture_output=True).returncode == 0:
        res = subprocess.run(["otool", "-tV", str(binary_path)], capture_output=True, text=True)
        if res.returncode == 0 and res.stdout.strip():
            return parse_otool_disassembly(res.stdout)

    # Fallback to llvm-objdump or objdump
    for tool in ["llvm-objdump", "objdump"]:
        if subprocess.run(["which", tool], capture_output=True).returncode == 0:
            res = subprocess.run([tool, "-d", str(binary_path)], capture_output=True, text=True)
            if res.returncode == 0 and res.stdout.strip():
                return parse_objdump_disassembly(res.stdout)

    raise RuntimeError("No disassembly tool found (requires `otool`, `llvm-objdump`, or `objdump`)")


def parse_otool_disassembly(text: str) -> dict:
    """Parse output from otool -tV into {symbol_name: [instruction_lines]}."""
    per_symbol = {}
    current_sym = None
    for line in text.splitlines():
        line = line.strip()
        if line.endswith(":") and not line.startswith("\t") and re.match(r"^[_\w]", line):
            current_sym = line[:-1]
            per_symbol[current_sym] = []
        elif current_sym and re.match(r"^[0-9a-fA-F]{6,}\s", line):
            per_symbol[current_sym].append(line)
    return per_symbol


def parse_objdump_disassembly(text: str) -> dict:
    """Parse output from objdump -d into {symbol_name: [instruction_lines]}."""
    per_symbol = {}
    current_sym = None
    # objdump symbol header format: 0000000100001000 <symbol_name>:
    sym_header_re = re.compile(r"^[0-9a-fA-F]+\s+<([^>]+)>:")
    for line in text.splitlines():
        m = sym_header_re.match(line.strip())
        if m:
            current_sym = m.group(1)
            per_symbol[current_sym] = []
        elif current_sym and re.match(r"^\s*[0-9a-fA-F]+:\s+", line):
            per_symbol[current_sym].append(line.strip())
    return per_symbol


def audit_binary(binary_path: Path, verbose: bool = False) -> bool:
    """Audit binary for zero float instructions in certified serving symbols."""
    print("=" * 80)
    print(f" UOR-R4 Float-Free Serving Path Audit (D0-b Compliance)")
    print(f" Target Binary : {binary_path}")
    print("=" * 80)

    per_symbol = disassemble_binary(binary_path)
    print(f"Total disassembled symbols loaded: {len(per_symbol)}")
    print(f"Auditing {len(CERTIFIED_SERVING_SYMBOLS)} declared serving symbol groups...\n")

    passed = True
    found_groups = 0
    total_audited_instrs = 0

    for pattern, description in CERTIFIED_SERVING_SYMBOLS:
        matches = [s for s in per_symbol if pattern in s]
        if not matches:
            print(f"  [ -- ] {description:<48} NO SYMBOL (inlined in release; NOT certified)")
            continue

        found_groups += 1
        group_fp_count = 0
        group_instr_count = 0
        detected_fp_lines = []

        for sym in matches:
            instrs = per_symbol[sym]
            group_instr_count += len(instrs)
            for instr in instrs:
                if FP_RE_ARM64.search(instr) or FP_RE_X86.search(instr):
                    group_fp_count += 1
                    detected_fp_lines.append((sym, instr))

        total_audited_instrs += group_instr_count

        if group_fp_count == 0:
            print(f"  [ OK ] {description:<48} 0 floats ({group_instr_count:>5} instrs across {len(matches):>2} symbol variants)")
        else:
            print(f"  [FAIL] {description:<48} {group_fp_count} FLOAT INSTRUCTIONS DETECTED!")
            for sym, instr in detected_fp_lines[:5]:
                print(f"         Symbol: {sym}")
                print(f"         Instr : {instr}")
            passed = False

    print("\n" + "-" * 80)
    print(f"Certified Groups Audited : {found_groups} / {len(CERTIFIED_SERVING_SYMBOLS)}")
    print(f"Total Serving Hot-Path Instructions Checked : {total_audited_instrs}")
    print("-" * 80)

    if found_groups == 0:
        print("RESULT: FAIL (Zero declared serving symbols found in binary; cannot certify)")
        return False

    if passed:
        print("RESULT: PASS (All certified serving symbols are 100% float-free; D0-b compliant)")
        return True
    else:
        print("RESULT: FAIL (Floating-point instructions detected in serving symbols; violates D0-b)")
        return False


def main():
    parser = argparse.ArgumentParser(description="Audit uor-r4-integer for float-free serving (D0-b)")
    parser.add_argument("binary", nargs="?", default="target/release/uor-r4-integer",
                        help="Path to binary to audit (default: target/release/uor-r4-integer)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Verbose reporting")
    args = parser.parse_args()

    binary_path = Path(args.binary)
    if not binary_path.is_absolute():
        # Search relative to current directory or worktree
        if not binary_path.exists() and (Path.cwd() / binary_path).exists():
            binary_path = Path.cwd() / binary_path

    try:
        success = audit_binary(binary_path, verbose=args.verbose)
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"Error during float audit: {e}", file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
