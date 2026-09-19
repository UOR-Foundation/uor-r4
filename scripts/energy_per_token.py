#!/usr/bin/env python3
"""Measure joules per token for a local serving command on Apple Silicon.

WHY A SCRIPT AND NOT A ONE-LINER
--------------------------------
`powermetrics` reports SoC-wide power, so a naive reading during a workload charges the model for
the machine's idle floor as well. On an M1 laptop that floor is a few watts, the same order as the
workload, so a raw number is not a J/token figure -- it is a wrong one. This script:

  1. samples an IDLE baseline first, with no workload running;
  2. samples during the workload you supply, aligned to its own start and end;
  3. reports both, and the difference, so the subtraction is visible rather than implied;
  4. divides by the token count, which it AUTO-DETECTS from the command's own output when it can.

THE COMMAND GOES AFTER A BARE `--`
---------------------------------
    sudo python3 scripts/energy_per_token.py --label "native" --idle-seconds 8 \
        -- ./target/release/r4-native-chat --model <artifact.rgm> --temperature 0.0 --top-k 1 \
           -- "Once upon a time there was a little girl who"

Everything after the first `--` is the command, verbatim. (An earlier version of this script
declared an argument named `--command` while documenting `--`; that mismatch is fixed.)

TOKEN COUNT
-----------
`--tokens` is optional. If omitted, the script looks for the count in the command's stdout:
`[N model tokens]` (the native CLI) or `eval count: N` (ollama). If neither is found it refuses to
report J/token rather than dividing by a guess -- an energy figure with a wrong denominator is
worse than no figure. Pass `--tokens N` to override.

REQUIREMENTS
------------
  * root, because `powermetrics` requires it.
  * a constant machine state across both phases: AC power, lid open, nobody using the machine.
    Battery-discharge power is not comparable with AC power, and a hot SoC is not comparable with
    a cool one.

WHAT IT CANNOT DO
-----------------
  * No per-process attribution. That is `powermetrics --show-process-energy`, a different
    measurement (per-process energy impact, not SoC power).
  * No control of SoC power state. Keep runs short and repeat them.
  * A single repeat is not a result. Run each configuration at least three times and report the
    range, not just the mean.
"""

import argparse
import math
import re
import statistics
import subprocess
import sys
import time

# `Combined Power` is the Apple Silicon total; the others are fallbacks for OS versions that name
# the field differently.
POWER_RE = re.compile(
    r"(?:Combined Power \(CPU \+ GPU \+ ANE\)|Package Power|CPU Power)\s*:\s*([0-9.]+)\s*mW"
)

# Token-count patterns read from the command's own output.
TOKEN_PATTERNS = [
    # r4-native-chat emits `[128 model tokens; length; 617.3 tok/s, 1619.9 us/token]`, so the
    # closing bracket does NOT follow the word "tokens". Verified against real output.
    re.compile(r"\[(\d+)\s+model tokens"),
    re.compile(r"eval count:\s*(\d+)"),  # ollama --verbose
    re.compile(r"\beval_count[\"'\s:]+(\d+)"),  # ollama API JSON
]


def split_on_double_dash(argv):
    """Return (options, command). Everything after the first `--` is the command verbatim."""
    if "--" in argv:
        i = argv.index("--")
        return argv[:i], argv[i + 1 :]
    return argv, []


def run_powermetrics(seconds, interval_ms):
    """Sample for `seconds` and return (readings_mW, raw_text, stderr)."""
    count = max(1, math.ceil(seconds * 1000 / interval_ms))
    cmd = ["powermetrics", "--samplers", "cpu_power", "-i", str(interval_ms), "-n", str(count)]
    out = subprocess.run(cmd, capture_output=True, text=True)
    readings = [float(m) for m in POWER_RE.findall(out.stdout)]
    return readings, out.stdout, out.stderr


def detect_tokens(text):
    for pat in TOKEN_PATTERNS:
        m = pat.search(text)
        if m:
            return int(m.group(1))
    return None


def main():
    options, command = split_on_double_dash(sys.argv[1:])

    ap = argparse.ArgumentParser(
        description="Joules per token via powermetrics, with an idle-baseline subtraction. "
        "The command to measure goes after a bare --."
    )
    ap.add_argument("--label", required=True, help="name of the configuration under test")
    ap.add_argument("--tokens", type=int, default=None, help="override the auto-detected count")
    ap.add_argument("--idle-seconds", type=float, default=8.0)
    ap.add_argument("--interval-ms", type=int, default=500)
    ap.add_argument(
        "--max-seconds",
        type=float,
        default=180.0,
        help="safety bound on the workload sampling window",
    )
    args = ap.parse_args(options)

    if not command:
        sys.exit(
            "no command supplied. Put it after a bare --, e.g.\n"
            "  sudo python3 scripts/energy_per_token.py --label X --idle-seconds 8 -- "
            "./target/release/r4-native-chat --model ... -- \"prompt\""
        )

    print(f"config : {args.label}")
    print(f"command: {' '.join(command)}")
    print()

    print(f"[1/2] idle baseline, {args.idle_seconds:.0f}s, no workload ...")
    idle, _, idle_err = run_powermetrics(args.idle_seconds, args.interval_ms)
    if not idle:
        sys.exit(
            "powermetrics produced no readings for the idle phase, so the subtraction is "
            "impossible and any J/token would be wrong.\n"
            f"stderr: {idle_err.strip()[:500]}\n"
            "If this says 'must be run as root', run the script under sudo."
        )
    idle_mean = statistics.mean(idle)
    print(f"      idle  n={len(idle):>3}  mean {idle_mean:8.1f} mW")
    print()

    print("[2/2] workload ...")
    # Run the sampler without -n so it lives until we terminate it, and bound it with -n computed
    # from --max-seconds in case a sample count is required on this OS.
    n = max(1, math.ceil(args.max_seconds * 1000 / args.interval_ms))
    sampler = subprocess.Popen(
        ["powermetrics", "--samplers", "cpu_power", "-i", str(args.interval_ms), "-n", str(n)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    t0 = time.perf_counter()
    run = subprocess.run(command, capture_output=True, text=True)
    elapsed = time.perf_counter() - t0
    sampler.terminate()
    try:
        sampler_out, sampler_err = sampler.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        sampler.kill()
        sampler_out, sampler_err = sampler.communicate()

    work = [float(m) for m in POWER_RE.findall(sampler_out or "")]
    if not work:
        print(f"      workload exited {run.returncode} in {elapsed:.2f}s")
        sys.exit(
            "powermetrics produced no readings for the workload phase.\n"
            f"stderr: {(sampler_err or '').strip()[:500]}\n"
            "If the workload finished in far less than one sampling interval "
            f"({args.interval_ms} ms), make it longer: the sampler cannot resolve a run shorter "
            "than its own window."
        )
    work_mean = statistics.mean(work)

    combined = (run.stdout or "") + (run.stderr or "")
    tokens = args.tokens if args.tokens else detect_tokens(combined)
    if not tokens:
        print(f"      workload exited {run.returncode} in {elapsed:.2f}s")
        sys.exit(
            "could not determine the token count from the command's output, and J/token with a "
            "guessed denominator is worse than no figure.\n"
            "Pass --tokens N using the count the tool itself reported.\n"
            "Last 400 bytes of the command's output:\n  " + combined[-400:].replace("\n", "\n  ")
        )

    print(f"      workload exited {run.returncode} in {elapsed:.2f}s")
    print(f"      work  n={len(work):>3}  mean {work_mean:8.1f} mW")
    print(f"      tokens detected: {tokens}{' (from --tokens)' if args.tokens else ' (auto)'}")
    print()

    gross_j = work_mean / 1000.0 * elapsed
    net_w = max(work_mean - idle_mean, 0.0)
    net_j = net_w / 1000.0 * elapsed

    print("RESULT")
    print(f"  wall time                 : {elapsed:.3f} s")
    print(f"  tokens                    : {tokens}")
    print(f"  decode rate               : {tokens / elapsed:.2f} tok/s")
    print(f"  idle power (not the model): {idle_mean:.1f} mW")
    print(f"  workload power            : {work_mean:.1f} mW")
    print(f"  GROSS energy              : {gross_j:.3f} J   ({gross_j / tokens:.4f} J/token)")
    print(f"  NET energy (idle removed) : {net_j:.3f} J   ({net_j / tokens:.4f} J/token)")
    print()
    print("Quote the NET figure: the gross one charges the model for the machine's idle floor.")
    print("A single run is not a result. Repeat at least three times and report the range.")
    print("Record: AC or battery, whether the SoC was warm, and where the token count came from.")
    if run.returncode != 0:
        print(f"\nwarning: the command exited {run.returncode}; timing may be meaningless.")
        print((run.stderr or "").strip()[:300])
    return 0


if __name__ == "__main__":
    sys.exit(main())
