#!/usr/bin/env python3
"""Measure joules per token for a local serving command on Apple Silicon.

WHY A SCRIPT AND NOT A ONE-LINER
--------------------------------
`powermetrics` reports system-wide package power, so a naive reading of it during a workload
charges the model for the machine's idle floor as well. On an M1 laptop that floor is a few
watts, which is the same order as the workload itself, so a raw number is not a J/token figure —
it is a wrong one. This script therefore:

  1. samples an IDLE baseline first, with no workload running;
  2. samples during the workload, aligned to the workload's own start and end;
  3. reports both, and the difference, so the subtraction is visible rather than implied;
  4. divides by the token count you supply, giving J/token.

It prints the raw figures as well as the corrected ones so nothing is hidden behind arithmetic.

REQUIREMENTS
------------
  * root, because `powermetrics` requires it:  sudo python3 scripts/energy_per_token.py ...
  * a state held constant across both phases: run on AC power with the lid open, and do not
    touch the machine during a measurement. Battery-discharge power is not comparable with
    AC-power numbers, and neither is a machine with a hot SoC against a cool one.

USAGE
-----
    sudo python3 scripts/energy_per_token.py \
        --label "qwen2.5:1.5b Q4 CPU" --tokens 96 --idle-seconds 8 \
        -- ollama run qwen2.5:1.5b "Write three short sentences about a small robot."

    sudo python3 scripts/energy_per_token.py \
        --label "uor-r4 native release" --tokens 128 --idle-seconds 8 \
        -- ./target/release/r4-native-chat --model <artifact.rgm> --temperature 0.0 --top-k 1 \
           -- "Once upon a time there was a little girl who"

`--tokens` must be the number of tokens the command actually generates (each tool reports it:
`ollama --verbose` prints `eval count`; the native CLI prints `[N model tokens]`). Measuring
energy without knowing the token count gives you joules, not joules per token.

WHAT IT PARSES
--------------
`powermetrics --samplers cpu_power` emits a block per sample. The line used is
`Combined Power (CPU + GPU + ANE): N mW` when present, else `Package Power: N mW`, else the
`CPU Power:` line, so the script still reports something on OS versions that name the field
differently. Every matched line in a phase is averaged.

WHAT IT CANNOT DO
-----------------
  * It cannot attribute energy to one process. `powermetrics --show-process-energy` can, if you
    want that instead; it is a different measurement (per-process energy impact, not SoC power).
  * It does not control the SoC's power state. On a fanless M1 a long run will throttle, so keep
    runs short and repeat them rather than concatenating.
  * A single repeat is not a result. Run each configuration at least three times and report the
    range, not the mean alone.
"""

import argparse
import re
import statistics
import subprocess
import sys
import time

POWER_RE = re.compile(
    r"(?:Combined Power \(CPU \+ GPU \+ ANE\)|Package Power|CPU Power)\s*:\s*([0-9.]+)\s*mW"
)


def sample_power(seconds, interval_ms):
    """Run powermetrics for `seconds` and return the list of sampled power readings in mW."""
    count = max(1, int(seconds * 1000 / interval_ms))
    cmd = [
        "powermetrics",
        "--samplers",
        "cpu_power",
        "-i",
        str(interval_ms),
        "-n",
        str(count),
    ]
    out = subprocess.run(cmd, capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(
            f"powermetrics failed (exit {out.returncode}); it must run as root.\n"
            f"stderr: {out.stderr.strip()[:400]}"
        )
    readings = [float(m) for m in POWER_RE.findall(out.stdout)]
    if not readings:
        sys.exit(
            "no power readings parsed; inspect the raw output below and adjust POWER_RE.\n"
            + out.stdout[:800]
        )
    return readings


def main():
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument("--label", required=True, help="name of the configuration under test")
    ap.add_argument("--tokens", type=int, required=True, help="tokens actually generated")
    ap.add_argument("--idle-seconds", type=float, default=8.0)
    ap.add_argument("--interval-ms", type=int, default=500)
    ap.add_argument("--command", nargs=argparse.REMAINDER, required=True)
    args = ap.parse_args()

    command = args.command
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        sys.exit("no command after --")

    print(f"config : {args.label}")
    print(f"command: {' '.join(command)}")
    print()

    print(f"[1/2] idle baseline, {args.idle_seconds:.0f}s, no workload ...")
    idle = sample_power(args.idle_seconds, args.interval_ms)
    idle_mean = statistics.mean(idle)
    print(f"      idle  n={len(idle):>3}  mean {idle_mean:8.1f} mW")
    print()

    print("[2/2] workload ...")
    sampler = subprocess.Popen(
        [
            "powermetrics",
            "--samplers",
            "cpu_power",
            "-i",
            str(args.interval_ms),
            "-n",
            "100000",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    t0 = time.perf_counter()
    run = subprocess.run(command, capture_output=True, text=True)
    elapsed = time.perf_counter() - t0
    sampler.terminate()
    try:
        stdout, _ = sampler.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        sampler.kill()
        stdout, _ = sampler.communicate()

    work = [float(m) for m in POWER_RE.findall(stdout or "")]
    if not work:
        sys.exit("no power readings during the workload; rerun and inspect powermetrics output.")
    work_mean = statistics.mean(work)

    print(f"      workload exited {run.returncode} in {elapsed:.2f}s (stdout {len(run.stdout)} bytes)")
    print(f"      work  n={len(work):>3}  mean {work_mean:8.1f} mW")
    print()

    gross_j = work_mean / 1000.0 * elapsed
    net_w = max(work_mean - idle_mean, 0.0)
    net_j = net_w / 1000.0 * elapsed
    tokens = max(args.tokens, 1)

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
    print("Record: AC or battery, ambient, whether the SoC was warm, and the token count source.")
    if run.returncode != 0:
        print(f"\nwarning: the command exited {run.returncode}; timing may be meaningless.")
        print(run.stderr.strip()[:300])
    return 0


if __name__ == "__main__":
    sys.exit(main())
