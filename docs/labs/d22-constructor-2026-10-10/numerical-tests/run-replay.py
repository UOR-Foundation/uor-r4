"""Saved numerical replay only; no model, gradients, or panel evaluation."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import time

root = Path(sys.argv[1]).resolve()
attempt = root / sys.argv[2]
attempt.mkdir(exist_ok=False)
source_root = Path(os.environ.get("D22_NUMERICAL_SOURCE_ROOT", str(root))).resolve()
source = source_root / "docs/labs/d22-constructor-2026-10-10/numerical-tests"
manifest = source / "source-manifest.json"
jobs = int(os.environ.get("D22_NUMERICAL_BUILD_JOBS", "2"))
address_limit = int(os.environ.get("D22_NUMERICAL_ADDRESS_LIMIT", str(6 * 1024**3)))
assert jobs in (1, 2) and 0 < address_limit <= 6 * 1024**3


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


for name, entry in json.loads(manifest.read_text())["files"].items():
    assert sha(source_root / name) == entry["sha256"], name
env = os.environ.copy()
env.update(CARGO_TARGET_DIR=str(root / "numerical-target"), CARGO_BUILD_JOBS=str(jobs),
           CARGO_INCREMENTAL="0", CARGO_PROFILE_RELEASE_DEBUG="0",
           OMP_NUM_THREADS=str(jobs), UOR_MICROLP_PROGRESS="1",
           D22_BASIS_ADMISSION_ROOT=str(root / "qualification-attempt2"))
record = {"schema": "uor-r4.d22-saved-replay-process/1",
          "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "source_manifest_sha256": sha(manifest), "runner_sha256": sha(Path(__file__)),
          "basis_attempt": "qualification-attempt2", "build_jobs": jobs,
          "source_root": str(source_root), "progress_logging": True,
          "address_space_limit_bytes": address_limit, "operations": []}


def save():
    (attempt / "process.json").write_text(json.dumps(record, indent=2) + "\n")


def limit():
    resource.setrlimit(resource.RLIMIT_AS, (address_limit, address_limit))


def run(name, argv):
    item = {"name": name, "command": argv,
            "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat()}
    record["operations"].append(item)
    save()
    start = time.monotonic()
    with (attempt / (name + ".log")).open("wb") as log:
        p = subprocess.run(argv, cwd=root, env=env, stdout=log,
                           stderr=subprocess.STDOUT, preexec_fn=limit)
    item.update(exit_code=p.returncode, elapsed_seconds=time.monotonic() - start,
                peak_child_rss_bytes_cumulative_max=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss * 1024,
                log_sha256=sha(attempt / (name + ".log")))
    save()
    return p.returncode


cargo = ["/root/.cargo/bin/cargo"]
common = ["--offline", "--release", "--manifest-path", str(source / "Cargo.toml"),
          "--bin", "d22-numerical-fixtures"]
for name, argv in [("synthetic-tests", cargo + ["test", "--offline", "--release", "--manifest-path",
                                             str(source / "Cargo.toml"), "--lib", "--", "--test-threads=1"]),
                   ("gate-tests", cargo + ["test"] + common + ["replay_gate_", "--", "--test-threads=1"]),
                   ("build", cargo + ["build"] + common)]:
    if run(name, argv):
        record["status"] = name.upper() + "_FAILED"
        save()
        sys.exit(1)
binary = root / "numerical-target/release/d22-numerical-fixtures"
record.update(binary_sha256=sha(binary), cargo_lock_sha256=sha(source / "Cargo.lock"))
code = run("constructor", [str(binary), "--input", str(root / "restored/input.json"),
           "--expected-sha256", "0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d",
           "--output", str(attempt / "constructor")])
record["status"] = "SAVED_REPLAY_COMPLETED_INSPECT_RECEIPT" if code == 0 else "SAVED_REPLAY_PROCESS_FAILED"
save()
print(json.dumps(record), flush=True)
sys.exit(code)
